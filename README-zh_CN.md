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
# 默认功能（包含 SHA2 和 SHA3）
robust_downloader = "0.0.6"

# 或者指定特定的哈希算法
robust_downloader = { version = "0.0.6", features = ["sha2", "blake3"] }

# 或者使用现代/安全的算法
robust_downloader = { version = "0.0.6", features = ["modern"] }

# 或者启用所有哈希算法
robust_downloader = { version = "0.0.6", features = ["all"] }
```

### 示例

```rust
use robust_downloader::{RobustDownloader, Integrity, DownloadItem};
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
        DownloadItem::builder()
            .url("https://example.com/file1.zip")
            .target("file1.zip")
            .integrity(Integrity::SHA256("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into()))
            .build(),
        DownloadItem::builder()
            .url("https://example.com/file2.zip")
            .target("file2.zip")
            .integrity(Integrity::Blake3("202020202020202020202020".into()))
            .build(),
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

## 哈希算法特性

可用的哈希算法特性：
- `md5` - 启用 MD5 哈希支持
- `sha1` - 启用 SHA1 哈希支持
- `sha2` - 启用 SHA256 和 SHA512 支持（默认包含）
- `sha3` - 启用 SHA3-256 哈希支持（默认包含）
- `blake2` - 启用 BLAKE2b 和 BLAKE2s 支持
- `blake3` - 启用 BLAKE3 哈希支持

特性组合：
- `modern` - 启用现代/安全算法（sha2、sha3、blake2、blake3）
- `legacy` - 启用传统算法（md5、sha1）
- `all` - 启用所有哈希算法

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