use std::{path::Path, time::Duration};

use futures::StreamExt;
use hashery::Hashery;
use indicatif::ProgressBar;
use reqwest::IntoUrl;
use tokio::io::AsyncWriteExt;
use typed_builder::TypedBuilder;

use crate::{err::ProgressDownloadError, item::DownloadItem, tracker::DownloadTracker};

#[derive(Debug, TypedBuilder)]
pub struct DownloadTasker<U: IntoUrl + Clone, P: AsRef<Path>, TP: AsRef<Path>> {
  #[builder]
  client: reqwest::Client,
  #[builder]
  progress_bar: ProgressBar,

  #[builder]
  tmp_file: P,

  #[builder]
  item: DownloadItem<U, TP>,
  #[builder]
  timeout: Duration,
  #[builder]
  flush_threshold: usize,
}

impl<U: IntoUrl + Clone, P: AsRef<Path>, TP: AsRef<Path>> DownloadTasker<U, P, TP> {
  async fn send(&self, downloaded_size: u64) -> Result<reqwest::Response, ProgressDownloadError> {
    let request = self
      .client
      .get(self.item.url.as_str())
      .header("Range", format!("bytes={}-", downloaded_size))
      .timeout(self.timeout);

    let response = request.send().await?;

    Ok(response)
  }

  pub async fn download(&self) -> Result<(), ProgressDownloadError> {
    let temp_file = self.tmp_file.as_ref();
    let downloaded_size = temp_file.metadata().map(|item| item.len()).unwrap_or(0);

    let response = self.send(downloaded_size).await?;
    let supports_resume = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    let remaining_size = response.content_length().unwrap_or(0);

    let should_resume = supports_resume && downloaded_size > 0;

    let file = tokio::fs::OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(!should_resume)
      .append(should_resume)
      .open(temp_file)
      .await?;

    let mut delegate = DownloadTracker::builder()
      .progress_bar(&self.progress_bar)
      .downloaded_size(downloaded_size)
      .remaining_size(remaining_size)
      .url(self.item.url.clone())
      .build();

    delegate.init_progress();

    let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1024, file);

    let stream = response.bytes_stream();

    tokio::pin!(stream);

    while let Some(chunk) = tokio::time::timeout(Duration::from_millis(500), stream.next())
      .await?
      .transpose()?
    {
      delegate.update_progress(chunk.len());

      writer.write_all(&chunk).await?;

      // 减少刷新频率，提高性能
      if writer.buffer().len() >= self.flush_threshold {
        writer.flush().await?;
      }
    }

    // 确保所有数据都写入
    writer.flush().await?;

    if let Some(integrity) = &self.item.integrity {
      let actual = Hashery::builder()
        .algorithm(integrity.algorithm())
        .build()
        .digest(temp_file)
        .await?;

      let expect = integrity.value().to_string();

      if actual != expect {
        tokio::fs::remove_file(temp_file).await?;
        return Err(ProgressDownloadError::IntegrityHash { expect, actual });
      }
    }

    // 确保目标文件的父目录存在
    if let Some(parent) = self.item.target.as_ref().parent() {
      tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::rename(&self.tmp_file, self.item.target.as_ref()).await?;

    Ok(())
  }
}
