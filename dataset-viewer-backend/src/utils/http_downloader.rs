use crate::storage::ProgressInfo;
use reqwest::Client;
use std::io::Write;
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// HTTP 下载器，支持进度回调和断点续传
pub struct HttpDownloader {
    client: Client,
}

impl HttpDownloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 下载文件到指定路径，支持进度回调
    pub async fn download_with_progress<F>(
        &self,
        url: &str,
        save_path: &Path,
        mut progress_callback: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(ProgressInfo) + Send,
    {
        // 发送 HEAD 请求获取文件大小
        let response = self.client.head(url).send().await?;
        let total_size = response
            .headers()
            .get("content-length")
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse::<u64>().ok())
            .unwrap_or(0);

        // 发送 GET 请求开始下载
        let mut response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP 错误: {}", response.status()).into());
        }

        // 创建文件
        let mut file = tokio::fs::File::create(save_path).await?;
        let mut downloaded = 0u64;

        // 分块下载
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // 调用进度回调
            let percentage = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            progress_callback(ProgressInfo {
                current: downloaded,
                total: total_size,
                percentage,
                speed: None, // TODO: 计算下载速度
                eta: None,   // TODO: 计算剩余时间
            });
        }

        file.sync_all().await?;
        Ok(())
    }

    /// 简单下载文件
    pub async fn download(
        &self,
        url: &str,
        save_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.download_with_progress(url, save_path, |_| {}).await
    }

    /// 下载文件内容到内存
    pub async fn download_bytes(&self, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        let response = self.client.get(url).send().await?;
        response.bytes().await.map(|b| b.to_vec())
    }

    /// 下载文件的指定范围
    pub async fn download_range(
        &self,
        url: &str,
        start: u64,
        end: u64,
    ) -> Result<Vec<u8>, reqwest::Error> {
        let response = self
            .client
            .get(url)
            .header("Range", format!("bytes={}-{}", start, end))
            .send()
            .await?;

        response.bytes().await.map(|b| b.to_vec())
    }
}