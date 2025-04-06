# Robust Downloader

[![Crates.io](https://img.shields.io/crates/v/robust_downloader.svg)](https://crates.io/crates/robust_downloader)
[![Documentation](https://docs.rs/robust_downloader/badge.svg)](https://docs.rs/robust_downloader)
[![License](https://img.shields.io/crates/l/robust_downloader.svg)](LICENSE)

一个强大的 Rust 并发文件下载库，具有进度跟踪和重试功能。

[English](README.md) | [中文](README-zh_CN.md)

## 特性

- 🚀 **并发下载**：支持同时下载多个文件，可配置并发限制
- 🔄 **自动重试**：内置指数退避重试机制，自动处理下载失败
- 📊 **进度跟踪**：美观的进度条，实时显示下载状态和统计信息
- ⚡ **性能优化**：高效的内存使用，可配置缓冲区大小
- 🛡️ **安全文件处理**：使用临时文件确保原子操作
- 🔒 **完整性验证**：支持多种哈希算法的文件完整性校验
- ⚙️ **高度可配置**：可自定义超时、并发数和重试行为

## 快速开始

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
robust_downloader = "0.0.5"
```

### 示例

```rust
use robust_downloader::{RobustDownloader, Integrity, HashAlgorithm};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建下载器并自定义配置
    let downloader = RobustDownloader::builder()
        .max_concurrent(4)                    // 设置最大并发数
        .connect_timeout(Duration::from_secs(5))  // 设置连接超时
        .build();

    // 定义下载任务，支持完整性验证
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

    // 开始下载
    downloader.download(downloads).await?;
    Ok(())
}
```

## 配置选项

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `max_concurrent` | 2 | 最大并发下载数 |
| `connect_timeout` | 2秒 | 每个请求的连接超时时间 |
| `timeout` | 60秒 | 每个下载的总超时时间 |
| `flush_threshold` | 512KB | 写入磁盘的缓冲区大小 |

## 进度跟踪

库提供了详细的进度跟踪功能：
- 下载进度百分比
- 当前下载文件名
- 不同阶段的状态信息（下载中、验证完整性、移动文件）
- 实时下载速度

## 安装

该库需要 Rust 1.75 或更高版本。

```bash
cargo add robust_downloader
```

## 开源协议

本项目采用 MIT 协议 - 查看 [LICENSE](LICENSE) 文件了解详情。 