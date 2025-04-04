use std::{env, sync::Arc, time::Duration};

use backoff::ExponentialBackoff;
use err::ProgressDownloadError;
use indicatif::{ProgressBar, ProgressDrawTarget};
use task::DownloadTasker;
use tokio::sync::Semaphore;
use typed_builder::TypedBuilder;

mod err;
mod task;
mod tracker;

#[derive(Debug, TypedBuilder, Clone)]
pub struct RobustDownloader {
  #[builder(default = Duration::from_millis(2_000))]
  connect_timeout: Duration,

  #[builder(default = Duration::from_secs(60))]
  timeout: Duration,

  // 添加新的配置参数，默认为 512KB
  #[builder(default = 512 * 1024)]
  flush_threshold: usize,

  #[builder(default = 2)]
  max_concurrent: usize,
}

impl RobustDownloader {
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

  pub async fn download(&self, downloads: Vec<(&str, &str)>) -> Result<(), ProgressDownloadError> {
    let client = reqwest::Client::builder()
      .connect_timeout(self.connect_timeout)
      .pool_max_idle_per_host(0)
      .build()?;

    let mp = indicatif::MultiProgress::new();

    // 创建信号量来控制并发
    let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

    let futures = downloads.into_iter().map(|(url, path)| {
      let sem = semaphore.clone();
      let client = client.clone();
      let mp = mp.clone();

      async move {
        // 获取信号量许可
        let _permit = sem.acquire().await?;
        self.download_with_retry(&client, &mp, url, path).await
      }
    });

    futures::future::try_join_all(futures).await?;
    mp.set_move_cursor(true);
    mp.clear()?;
    println!("🎉 Download completed");

    Ok(())
  }

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

  async fn download_with_retry(
    &self,
    client: &reqwest::Client,
    mp: &indicatif::MultiProgress,
    url: &str,
    target: &str,
  ) -> Result<(), ProgressDownloadError> {
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(target);

    let progress_bar = self.prepare_progress_bar();
    let progress_bar = mp.add(progress_bar);

    let tasker = DownloadTasker::builder()
      .client(client.clone())
      .progress_bar(progress_bar)
      .url(url.to_string())
      .tmp_file(temp_file)
      .target_file(target)
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
