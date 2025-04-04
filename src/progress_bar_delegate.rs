use std::time::Instant;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct ProgressBarDelegate<'a> {
  #[builder]
  downloaded_size: u64,
  #[builder]
  remaining_size: u64,
  #[builder(default = Instant::now())]
  start_time: Instant,
  #[builder]
  url: String,
  #[builder]
  progress_bar: &'a indicatif::ProgressBar,
}

impl<'a> ProgressBarDelegate<'a> {
  pub fn init_progress(&mut self) {
    self
      .progress_bar
      .set_length(self.remaining_size + self.downloaded_size);
    self.progress_bar.set_position(self.downloaded_size);
  }

  pub fn update_progress(&mut self, chunk_size: usize) {
    self.downloaded_size += chunk_size as u64;
    self.progress_bar.set_position(self.downloaded_size);
    self.update_speed();
  }

  fn update_speed(&mut self) {
    let elapsed = self.start_time.elapsed().as_secs_f64();
    if elapsed > 0.0 {
      // let speed = (self.downloaded_size - self.last_downloaded_size) as f64
      //     / elapsed
      //     / 1024.0
      //     / 1024.0;
      let percentage = (self.downloaded_size as f64
        / (self.remaining_size + self.downloaded_size) as f64
        * 100.0) as u64;
      self
        .progress_bar
        .set_message(format!("{}% {} ", percentage, self.url));
      // self.last_downloaded_size = self.downloaded_size;
    }
  }
}
