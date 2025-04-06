use std::{env, path::Path, sync::Arc, time::Duration};

use backoff::ExponentialBackoff;
use err::ProgressDownloadError;
use indicatif::{ProgressBar, ProgressDrawTarget};
use reqwest::IntoUrl;
use task::DownloadTasker;
use tokio::sync::Semaphore;
use typed_builder::TypedBuilder;

mod err;
mod item;
mod task;
mod tracker;

pub use item::*;

/// A robust, concurrent file downloader with retry capabilities and progress tracking.
///
/// `RobustDownloader` provides a reliable way to download multiple files concurrently with features like:
/// - Automatic retries with exponential backoff
/// - Progress bars for visual feedback
/// - Concurrent downloads with configurable limits
/// - Timeouts and connection management
/// - Temporary file handling for safe downloads
///
/// # Example
///
/// ```rust
/// use robust_downloader::{RobustDownloader, DownloadItem};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let downloader = RobustDownloader::builder()
///         .max_concurrent(4)
///         .build();
///
///     let downloads = vec![
///         DownloadItem::builder()
///             .url("https://example.com/file1.zip")
///             .target("local/file1.zip")
///             .build(),
///         DownloadItem::builder()
///             .url("https://example.com/file2.zip")
///             .target("local/file2.zip")
///             .build(),
///     ];
///
///     downloader.download(downloads).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, TypedBuilder, Clone)]
pub struct RobustDownloader {
  /// Connection timeout for each download request.
  /// Defaults to 2 seconds.
  #[builder(default = Duration::from_millis(2_000))]
  connect_timeout: Duration,

  /// Overall timeout for each download operation.
  /// Defaults to 60 seconds.
  #[builder(default = Duration::from_secs(60))]
  timeout: Duration,

  /// Buffer size threshold for flushing downloaded data to disk.
  /// Defaults to 512KB.
  #[builder(default = 512 * 1024)]
  flush_threshold: usize,

  /// Maximum number of concurrent downloads.
  /// Defaults to 2.
  #[builder(default = 2)]
  max_concurrent: usize,
}

impl RobustDownloader {
  /// Creates an exponential backoff configuration for retry attempts.
  ///
  /// The configuration uses the following parameters:
  /// - Initial interval: 500ms
  /// - Randomization factor: 15%
  /// - Multiplier: 1.5x
  /// - Maximum interval: 5 seconds
  /// - Maximum elapsed time: 120 seconds
  fn backoff(&self) -> ExponentialBackoff {
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

  /// Downloads multiple files concurrently with progress tracking and retry capabilities.
  ///
  /// # Arguments
  ///
  /// * `downloads` - A vector of tuples containing (url, target_path) pairs.
  ///                 The URL specifies where to download from, and target_path is where to save the file.
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` if all downloads complete successfully, or a `ProgressDownloadError`
  /// if any download fails after all retry attempts.
  ///
  /// # Example
  ///
  /// ```rust
  /// use robust_downloader::{RobustDownloader, DownloadItem};
  /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let downloader = RobustDownloader::builder().build();
  /// let files = vec![
  ///     DownloadItem::builder()
  ///         .url("https://example.com/file1.txt")
  ///         .target("local/file1.txt")
  ///         .build(),
  ///     DownloadItem::builder()
  ///         .url("https://example.com/file2.txt")
  ///         .target("local/file2.txt")
  ///         .build(),
  /// ];
  /// downloader.download(files).await?;
  /// # Ok(())
  /// # }
  /// ```
  pub async fn download<U, P>(
    &self,
    downloads: Vec<DownloadItem<U, P>>,
  ) -> Result<(), ProgressDownloadError>
  where
    U: IntoUrl + Clone,
    P: AsRef<Path>,
  {
    let client = reqwest::Client::builder()
      .connect_timeout(self.connect_timeout)
      .pool_max_idle_per_host(0)
      .build()?;

    let mp = indicatif::MultiProgress::new();

    // 创建信号量来控制并发
    let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

    let futures = downloads.into_iter().map(|item| {
      let sem = semaphore.clone();
      let client = client.clone();
      let mp = mp.clone();

      async move {
        // 获取信号量许可
        let _permit = sem.acquire().await?;
        self.download_with_retry(&client, &mp, item).await
      }
    });

    futures::future::try_join_all(futures).await?;
    mp.set_move_cursor(true);
    mp.clear()?;

    Ok(())
  }

  /// Creates a new progress bar with a standardized style for download tracking.
  ///
  /// The progress bar includes:
  /// - A green spinner
  /// - Elapsed time
  /// - A 25-character wide progress bar
  /// - Downloaded bytes / Total bytes
  /// - Additional status messages
  fn prepare_progress_bar(&self) -> ProgressBar {
    let progress_bar = ProgressBar::with_draw_target(Some(0), ProgressDrawTarget::stdout());
    progress_bar.set_style(
            indicatif::ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {bar:25.green/white.dim} {bytes}/{total_bytes} {wide_msg:.dim}",
            )
            .unwrap()
            .progress_chars("━━"),
        );
    progress_bar
  }

  /// Attempts to download a single file with automatic retries on failure.
  ///
  /// This method implements the retry logic using exponential backoff and
  /// handles temporary file management to ensure atomic file operations.
  ///
  /// # Arguments
  ///
  /// * `client` - The HTTP client to use for the download
  /// * `mp` - Multi-progress bar for tracking multiple downloads
  /// * `url` - The URL to download from
  /// * `target` - The local path where the file should be saved
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` if the download succeeds, or a `ProgressDownloadError`
  /// if the download fails after all retry attempts.
  async fn download_with_retry<U, P>(
    &self,
    client: &reqwest::Client,
    mp: &indicatif::MultiProgress,
    item: DownloadItem<U, P>,
  ) -> Result<(), ProgressDownloadError>
  where
    U: IntoUrl + Clone,
    P: AsRef<Path>,
  {
    let target_file = item.target.as_ref();

    let Some(file_name) = target_file.file_name() else {
      return Err(ProgressDownloadError::Path {
        path: target_file.to_string_lossy().to_string(),
      });
    };

    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(file_name);

    let progress_bar = self.prepare_progress_bar();
    let progress_bar = mp.add(progress_bar);

    let tasker = DownloadTasker::builder()
      .client(client.clone())
      .progress_bar(progress_bar)
      .item(item)
      .tmp_file(temp_file)
      .timeout(self.timeout)
      .flush_threshold(self.flush_threshold)
      .build();

    backoff::future::retry(self.backoff(), || async {
      tasker
        .download()
        .await
        .map_err(ProgressDownloadError::into_backoff_err)
    })
    .await?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use hashery::Algorithm;

  use crate::item::IntegrityHash;

  use super::*;

  #[tokio::test]
  async fn test_download() {
    let downloader = RobustDownloader::builder()
      .connect_timeout(Duration::from_secs(1))
      .timeout(Duration::from_secs(60))
      .flush_threshold(1024 * 1024)
      .build();
    let downloads = vec![
      DownloadItem::builder()
        .url("https://nodejs.org/dist/v23.9.0/node-v23.9.0.tar.gz")
        .target("local/node-v23.9.0.tar.gz")
        .integrity(
          IntegrityHash::builder()
            .expect("164ec8fe82aac21f74efc0d5890d9f6c0e0ba22ca285d400c0266913fb4ff8a0".to_string())
            .algorithm(Algorithm::SHA256)
            .build(),
        )
        .build(),
    ];
    downloader.download(downloads).await.unwrap();
  }
}
