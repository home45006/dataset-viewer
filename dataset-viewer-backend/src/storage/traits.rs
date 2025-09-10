use async_trait::async_trait;
use std::sync::Arc;

use crate::storage::{
    ConnectionConfig, DirectoryResult, FileContent, ListOptions, ProgressInfo, StorageError,
};

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// 统一存储客户端接口
#[async_trait]
pub trait StorageClient: Send + Sync {
    /// 连接到存储服务
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), StorageError>;

    /// 检查是否已连接
    async fn is_connected(&self) -> bool;

    /// 断开连接
    async fn disconnect(&mut self) -> Result<(), StorageError>;

    /// 列出目录内容
    async fn list_directory(
        &self,
        path: &str,
        options: Option<&ListOptions>,
    ) -> Result<DirectoryResult, StorageError>;

    /// 读取文件的指定范围
    async fn read_file_range(
        &self,
        path: &str,
        start: u64,
        length: u64,
    ) -> Result<Vec<u8>, StorageError>;

    /// 读取文件的指定范围，支持进度回调
    async fn read_file_range_with_progress(
        &self,
        path: &str,
        start: u64,
        length: u64,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<u8>, StorageError> {
        // 默认实现：调用不带进度的版本
        let result = self.read_file_range(path, start, length).await?;
        
        if let Some(callback) = progress_callback {
            callback(ProgressInfo {
                current: length,
                total: length,
                percentage: 100.0,
                speed: None,
                eta: None,
            });
        }
        
        Ok(result)
    }

    /// 读取完整文件
    async fn read_full_file(&self, path: &str) -> Result<Vec<u8>, StorageError>;

    /// 读取完整文件，支持进度回调
    async fn read_full_file_with_progress(
        &self,
        path: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<u8>, StorageError> {
        // 默认实现：调用不带进度的版本
        let result = self.read_full_file(path).await?;
        
        if let Some(callback) = progress_callback {
            let size = result.len() as u64;
            callback(ProgressInfo {
                current: size,
                total: size,
                percentage: 100.0,
                speed: None,
                eta: None,
            });
        }
        
        Ok(result)
    }

    /// 获取文件大小
    async fn get_file_size(&self, path: &str) -> Result<u64, StorageError>;

    /// 获取文件内容（带元数据）
    async fn get_file_content(
        &self,
        path: &str,
        start: Option<u64>,
        length: Option<u64>,
    ) -> Result<FileContent, StorageError> {
        let data = match (start, length) {
            (Some(s), Some(l)) => self.read_file_range(path, s, l).await?,
            _ => self.read_full_file(path).await?,
        };

        Ok(FileContent {
            size: data.len() as u64,
            content: data,
            mime_type: None,
            encoding: Some("binary".to_string()),
        })
    }

    /// 获取下载 URL（对于需要签名的存储如 OSS）
    async fn get_download_url(&self, path: &str) -> Result<String, StorageError> {
        // 默认实现：直接返回路径
        Ok(path.to_string())
    }

    /// 获取协议名称
    fn protocol(&self) -> &str;

    /// 验证配置是否有效
    fn validate_config(&self, config: &ConnectionConfig) -> Result<(), StorageError>;

    /// 构建协议 URL
    fn build_protocol_url(&self, path: &str) -> String;
}