use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Stream;
use indicatif::ProgressBar;
use pin_project::{pin_project, pinned_drop};
use tokio::time::Instant;
use typed_builder::TypedBuilder;

#[pin_project(PinnedDrop)]
#[derive(TypedBuilder)]
pub struct ProgressBarStream {
    #[pin]
    inner: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    progress_bar: ProgressBar,
    #[builder(default = Instant::now())]
    start_time: Instant,
    current_size: u64,
    downloaded_size: u64,
    url: String,
    remaining_size: u64,
}

impl Stream for ProgressBarStream {
    type Item = Result<bytes::Bytes, reqwest::Error>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.inner.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                *this.current_size += chunk.len() as u64;
                this.progress_bar.set_position(*this.current_size);
                let elapsed = this.start_time.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let speed = (*this.current_size - *this.downloaded_size) as f64
                        / elapsed
                        / 1024.0
                        / 1024.0;
                    this.progress_bar
                        .set_message(format!("{} {:.2} MB/s", this.url, speed));
                }
                Poll::Ready(Some(Ok(chunk)))
            }
            other => {
                return other;
            }
        }
    }
}

#[pinned_drop]
impl PinnedDrop for ProgressBarStream {
    fn drop(self: Pin<&mut Self>) {
        // 安全地访问 progress_bar
        self.project().progress_bar.abandon();
    }
}
