use indicatif::ProgressBar;
use std::time::Instant;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct DownloadState {
  #[builder(default = 0)]
  downloaded_size: u64,
  #[builder(default = 0)]
  remaining_size: u64,
  // #[builder(default = 0)]
  // last_downloaded_size: u64,
  #[builder(default = Instant::now())]
  start_time: Instant,
  #[builder(default = String::new())]
  url: String,
  // #[builder(default = 0)]
  // total_size: u64,
}

impl DownloadState {
  pub fn update_progress(&mut self, chunk_size: usize, progress_bar: &ProgressBar) {
    self.downloaded_size += chunk_size as u64;
    progress_bar.set_position(self.downloaded_size);
    self.update_speed(progress_bar);
  }

  fn update_speed(&mut self, progress_bar: &ProgressBar) {
    let elapsed = self.start_time.elapsed().as_secs_f64();
    if elapsed > 0.0 {
      // let speed = (self.downloaded_size - self.last_downloaded_size) as f64
      //     / elapsed
      //     / 1024.0
      //     / 1024.0;
      let percentage = (self.downloaded_size as f64
        / (self.remaining_size + self.downloaded_size) as f64
        * 100.0) as u64;
      progress_bar.set_message(format!("{}% {} ", percentage, self.url));
      // self.last_downloaded_size = self.downloaded_size;
    }
  }
}
