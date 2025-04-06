# Robust Downloader

[![Crates.io](https://img.shields.io/crates/v/robust_downloader.svg)](https://crates.io/crates/robust_downloader)
[![Documentation](https://docs.rs/robust_downloader/badge.svg)](https://docs.rs/robust_downloader)
[![License](https://img.shields.io/crates/l/robust_downloader.svg)](LICENSE)

A robust, concurrent file downloader library for Rust with progress tracking and retry capabilities.

[English](README.md) | [中文](README-zh_CN.md)

## Features

- 🚀 **Concurrent Downloads**: Download multiple files simultaneously with configurable concurrency limits
- 🔄 **Automatic Retries**: Built-in exponential backoff retry mechanism for failed downloads
- 📊 **Progress Tracking**: Beautiful progress bars with real-time download statistics and status messages
- ⚡ **Performance Optimized**: Efficient memory usage with configurable buffer sizes
- 🛡️ **Safe File Handling**: Uses temporary files for atomic operations
- 🔒 **Integrity Verification**: Support for file integrity checking with various hash algorithms
- ⚙️ **Highly Configurable**: Customize timeouts, concurrency, and retry behavior

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
robust_downloader = "0.1.0"
```

### Example

```rust
use robust_downloader::{RobustDownloader, Integrity, HashAlgorithm};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a downloader with custom configuration
    let downloader = RobustDownloader::builder()
        .max_concurrent(4)                    // Set max concurrent downloads
        .connect_timeout(Duration::from_secs(5))  // Set connection timeout
        .build();

    // Define download tasks with integrity verification
    let downloads = vec![
        (
            "https://example.com/file1.zip",
            "file1.zip",
            Some(Integrity {
                algorithm: HashAlgorithm::SHA256,
                expect: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            })
        ),
        ("https://example.com/file2.zip", "file2.zip", None),
    ];

    // Start downloading
    downloader.download(downloads).await?;
    Ok(())
}
```

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `max_concurrent` | 2 | Maximum number of concurrent downloads |
| `connect_timeout` | 2s | Connection timeout for each request |
| `timeout` | 60s | Overall timeout for each download |
| `flush_threshold` | 512KB | Buffer size for writing to disk |

## Progress Tracking

The library provides detailed progress tracking with:
- Download progress percentage
- Current file being downloaded
- Status messages for different stages (downloading, verifying integrity, moving file)
- Real-time download speed

## Installation

The library requires Rust 1.75 or later.

```bash
cargo add robust_downloader
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 