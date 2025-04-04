use std::time::Duration;

use robust_downloader::RobustDownloader;

#[tokio::main]
async fn main() {
  let download_progress = RobustDownloader::builder()
    .connect_timeout(Duration::from_secs(1))
    .timeout(Duration::from_secs(60))
    .flush_threshold(1024 * 1024)
    .build();

  // download_progress
  //     .download(
  //         "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-x64.tar.gz",
  //         "node-v10.23.3-win-x86.7z",
  //     )
  //     .await
  //     .unwrap();

  let downloads = vec![
    (
      "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-x64.tar.gz",
      "node-v23.9.0-linux-x64.tar.gz",
    ),
    (
      "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-arm64.tar.gz",
      "node-v23.9.0-linux-arm64.tar.gz1",
    ),
    (
      "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-arm64.tar.gz",
      "node-v23.9.0-linux-arm64.tar.gz2",
    ),
    (
      "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-arm64.tar.gz",
      "node-v23.9.0-linux-arm64.tar.gz3",
    ),
  ];

  download_progress.download(downloads).await.unwrap();

  // let mut tasks = Vec::new();

  // for (url, path) in downloads {
  //     let download_progress = download_progress.clone();
  //     let task = tokio::spawn(async move {
  //         download_progress.download(url, path).await.unwrap();
  //     });
  //     tasks.push(task);
  // }

  // for task in tasks {
  //     task.await.unwrap();
  // }

  // download_progress
  //     .download_multiple(vec![
  //         (
  //             "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-x64.tar.gz",
  //             "node-v23.9.0-linux-x64.tar.gz",
  //         ),
  //         (
  //             "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-arm64.tar.gz",
  //             "node-v23.9.0-linux-arm64.tar.gz",
  //         ),
  //     ])
  //     .await
  //     .unwrap();
}
