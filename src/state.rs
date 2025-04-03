use indicatif::ProgressBar;
use std::time::Instant;

#[derive(Debug)]
pub struct DownloadState {
    downloaded_size: u64,
    last_downloaded_size: u64,
    start_time: Instant,
    url: String,
    total_size: u64,
}

impl DownloadState {
    pub fn new(url: String, initial_size: u64, total_size: u64) -> Self {
        Self {
            downloaded_size: initial_size,
            last_downloaded_size: initial_size,
            start_time: Instant::now(),
            url,
            total_size,
        }
    }

    pub fn update_progress(&mut self, chunk_size: usize, progress_bar: &ProgressBar) {
        self.downloaded_size += chunk_size as u64;
        progress_bar.set_position(self.downloaded_size);
        self.update_speed(progress_bar);
    }

    fn update_speed(&mut self, progress_bar: &ProgressBar) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            let speed = (self.downloaded_size - self.last_downloaded_size) as f64
                / elapsed
                / 1024.0
                / 1024.0;
            let percentage = (self.downloaded_size as f64 / self.total_size as f64 * 100.0) as u64;
            progress_bar.set_message(format!("{} {:.2} MB/s {}%", self.url, speed, percentage));
            self.last_downloaded_size = self.downloaded_size;
        }
    }
}
