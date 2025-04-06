use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProgressDownloadError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Reqwest error: {0}")]
  Reqwest(#[from] reqwest::Error),

  #[error("Timeout error: {0}")]
  Timeout(#[from] tokio::time::error::Elapsed),

  #[error("Semaphore error: {0}")]
  Semaphore(#[from] tokio::sync::AcquireError),

  #[error("Path error: {path}")]
  Path { path: String },

  #[error("Integrity hash error: {expect} != {actual}")]
  IntegrityHash { expect: String, actual: String },
}

impl ProgressDownloadError {
  fn is_retry_error(&self, e: &reqwest::Error) -> bool {
    // 1. 超时相关
    e.is_timeout() ||  // 请求超时
    e.is_connect() ||  // 连接错误
    e.is_request() ||  // 请求错误    
    // 3. 服务端错误 (5xx)
    (e.status().is_some_and(|s| s.is_server_error())) ||  // 500-599 服务器错误    
    // 4. 特定客户端错误
    (e.status().is_some_and(|s| {
      matches!(s.as_u16(),
        408 | // Request Timeout
        425 | // Too Early
        429 | // Too Many Requests
        449   // Retry With
      )
    })) ||
    // 5. 数据处理错误
    e.is_decode() ||      // 解码错误
    e.is_body() // 响应体错误
  }

  pub fn into_backoff_err(self) -> backoff::Error<Self> {
    match &self {
      Self::Io(err) => match err.kind() {
        // 1. 资源暂时不可用
        std::io::ErrorKind::WouldBlock |     // 操作会阻塞
        std::io::ErrorKind::Interrupted |    // 操作被中断
        std::io::ErrorKind::ResourceBusy |   // 资源正在被使用
        // 2. 网络相关
        std::io::ErrorKind::ConnectionReset |    // 连接被重置
        std::io::ErrorKind::ConnectionAborted |  // 连接中断
        std::io::ErrorKind::BrokenPipe |        // 管道破裂
        std::io::ErrorKind::TimedOut |          // 超时
        // 3. 系统资源相关
        std::io::ErrorKind::OutOfMemory |    // 内存不足（可能是临时的）
        std::io::ErrorKind::Other            // 其他未知错误（保守重试）
              => backoff::Error::transient(self),
        // 其他都是永久性错误
        _ => backoff::Error::permanent(self),
      },
      Self::Reqwest(error) => {
        if self.is_retry_error(error) {
          backoff::Error::transient(self)
        } else {
          backoff::Error::permanent(self)
        }
      }
      Self::Timeout(_) => backoff::Error::transient(self),
      Self::Semaphore(_) => backoff::Error::transient(self),
      Self::Path { .. } => backoff::Error::permanent(self),
      Self::IntegrityHash { .. } => backoff::Error::permanent(self),
    }
  }
}
