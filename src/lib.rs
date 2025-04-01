use std::{env, sync::Arc, time::Duration};

use backoff::ExponentialBackoff;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressDrawTarget};
use progress_bar_stream::ProgressBarStream;
use tokio::io::AsyncWriteExt;
use typed_builder::TypedBuilder;

mod progress_bar_stream;

pub trait Progress {
    fn create_progress(&self, url: &str, total_size: u64) -> ProgressBar;

    fn set_progress(&self, url: &str, downloaded_size: u64, remaining_size: u64);
}

#[derive(Debug, TypedBuilder, Clone)]
pub struct DownloadProgress {
    #[builder(default = Duration::from_millis(2_000))]
    connect_timeout: Duration,
    #[builder(default = Arc::new(indicatif::MultiProgress::new()))]
    multi_progress: Arc<indicatif::MultiProgress>,
    #[builder(default = Arc::new(
      reqwest::Client::builder()
          .connect_timeout(Duration::from_millis(2_000))
          .pool_max_idle_per_host(0)
          .build()
          .expect("Failed to create HTTP client")
  ))]
    client: Arc<reqwest::Client>,
    // progress: Box<dyn Progress>,
}

impl DownloadProgress {
    fn build_backoff(&self) -> ExponentialBackoff {
        ExponentialBackoff {
            // 初始等待 0.5 秒,加快重试速度
            initial_interval: Duration::from_millis(500),
            // 保持 15% 随机波动不变
            randomization_factor: 0.15,
            // 每次增加 1.5 倍,减缓增长速度
            multiplier: 1.5,
            // 最大等待 5 秒,缩短最大等待时间
            max_interval: Duration::from_secs(5),
            // 最多重试 1 分钟
            max_elapsed_time: Some(Duration::from_secs(120)),
            ..Default::default()
        }
    }

    fn is_retry_error(&self, e: &reqwest::Error) -> bool {
        e.is_timeout() || e.is_connect() || e.is_request() || e.is_decode()
    }

    pub async fn download_multiple(
        &self,
        downloads: Vec<(&'static str, &'static str)>,
    ) -> anyhow::Result<()> {
        let mut tasks = Vec::new();

        // self.multi_progress
        //     .set_alignment(indicatif::MultiProgressAlignment::Top);

        for (url, path) in downloads {
            let url = url.to_string();
            let path = path.to_string();

            // 使用Arc克隆指针，非常高效
            let client = self.client.clone();
            let multi_progress = self.multi_progress.clone();

            // 创建一个轻量级任务
            let task =
                tokio::spawn(
                    async move { self_download(&client, &multi_progress, &url, &path).await },
                );

            tasks.push(task);
        }

        // 等待所有下载任务完成
        for task in tasks {
            task.await??;
        }

        Ok(())
    }

    pub async fn download(&self, url: &str, path: &str) -> anyhow::Result<()> {
        // 直接使用self_download函数，避免代码重复
        self_download(&self.client, &self.multi_progress, url, path).await
    }
}

// 提取为独立函数，避免需要传递整个DownloadProgress实例
async fn self_download(
    client: &Arc<reqwest::Client>,
    multi_progress: &indicatif::MultiProgress,
    url: &str,
    path: &str,
) -> anyhow::Result<()> {
    // 创建指数退避配置
    let backoff = ExponentialBackoff {
        initial_interval: Duration::from_millis(500),
        randomization_factor: 0.15,
        multiplier: 1.5,
        max_interval: Duration::from_secs(5),
        max_elapsed_time: Some(Duration::from_secs(120)),
        ..Default::default()
    };

    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(path);

    let progress_bar = ProgressBar::with_draw_target(Some(0), ProgressDrawTarget::stdout());

    progress_bar.set_style(
        indicatif::ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:25.green/white.dim} {bytes}/{total_bytes} {wide_msg:.dim}",
        )
        .unwrap()
        .progress_chars("━━"),
    );

    let progress_bar = multi_progress.add(progress_bar);

    backoff::future::retry(backoff, || async {
        let downloaded_size = if temp_file.exists() {
            temp_file.metadata().unwrap().len()
        } else {
            0
        };

        let mut request = client.get(url);

        if downloaded_size > 0 {
            request = request.header("Range", format!("bytes={}-", downloaded_size));
        }

        let response = request.send().await.map_err(|e| {
            if is_retry_error(&e) {
                // 网络相关的临时错误，应该重试
                backoff::Error::transient(anyhow::Error::from(e))
            } else {
                // 其他错误视为永久错误
                backoff::Error::permanent(anyhow::Error::from(e))
            }
        })?;

        if !response.status().is_success() {
            return Err(backoff::Error::Permanent(anyhow::anyhow!(
                "Get request failed, Http status code not ok {} : {:?}",
                response.status(),
                response
            )));
        }

        let supports_resume = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;

        let file = if supports_resume && downloaded_size > 0 {
            tokio::fs::OpenOptions::new()
                .append(true)
                .open(temp_file.clone())
                .await
                .map_err(|e| backoff::Error::Permanent(anyhow::Error::from(e)))?
        } else {
            tokio::fs::File::create(temp_file.clone())
                .await
                .map_err(|e| backoff::Error::Permanent(anyhow::Error::from(e)))?
        };

        let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1024, file);

        let total_size = response.content_length();

        let stream = response.bytes_stream();

        progress_bar.set_length(total_size.unwrap_or(0) + downloaded_size);
        progress_bar.set_position(downloaded_size);

        let download_stream = ProgressBarStream::builder()
            .progress_bar(progress_bar.clone())
            .current_size(downloaded_size)
            .downloaded_size(downloaded_size)
            .url(url.to_string())
            .remaining_size(total_size.unwrap_or(0))
            .inner(Box::pin(stream))
            .build();

        tokio::pin!(download_stream);

        while let Some(chunk) =
            tokio::time::timeout(Duration::from_millis(500), download_stream.next())
                .await
                .map_err(|e| backoff::Error::transient(anyhow::Error::from(e)))?
                .transpose()
                .map_err(|e| {
                    if is_retry_error(&e) {
                        backoff::Error::transient(anyhow::Error::from(e))
                    } else {
                        backoff::Error::permanent(anyhow::Error::from(e))
                    }
                })?
        {
            writer
                .write_all(&chunk)
                .await
                .map_err(|e| backoff::Error::Permanent(anyhow::Error::from(e)))?;

            // 减少刷新频率，提高性能
            if writer.buffer().len() >= 512 * 1024 {
                writer
                    .flush()
                    .await
                    .map_err(|e| backoff::Error::Permanent(anyhow::Error::from(e)))?;
            }
        }

        // 确保所有数据都写入
        writer
            .flush()
            .await
            .map_err(|e| backoff::Error::Permanent(anyhow::Error::from(e)))?;

        progress_bar.finish_with_message(format!("Downloaded {} to {}", url, path));
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("Download failed after retries: {:#?}", e))?;

    tokio::fs::rename(&temp_file, path)
        .await
        .map_err(|e| anyhow::anyhow!("Rename failed: {}", e))?;

    Ok(())
}

// 提取为独立函数
fn is_retry_error(e: &reqwest::Error) -> bool {
    e.is_timeout() || e.is_connect() || e.is_request() || e.is_decode()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_download_progress() {
//         let download_progress = DownloadProgress::builder().build();

//         download_progress
//             .download(
//                 "https://code.visualstudio.com/sha/download?build=stable&os=win32-x64",
//                 "node-v10.23.3-win-x86.7z",
//             )
//             .await
//             .unwrap();

//         println!("{:?}", download_progress);
//     }
// }
