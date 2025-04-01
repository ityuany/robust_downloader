use progress_downloader::DownloadProgress;

#[tokio::main]
async fn main() {
    let download_progress = DownloadProgress::builder().build();

    download_progress
        .download(
            "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-x64.tar.gz",
            "node-v10.23.3-win-x86.7z",
        )
        .await
        .unwrap();

    download_progress
        .download_multiple(vec![
            (
                "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-x64.tar.gz",
                "node-v23.9.0-linux-x64.tar.gz",
            ),
            (
                "https://nodejs.org/dist/v23.9.0/node-v23.9.0-linux-arm64.tar.gz",
                "node-v23.9.0-linux-arm64.tar.gz",
            ),
        ])
        .await
        .unwrap();
}
