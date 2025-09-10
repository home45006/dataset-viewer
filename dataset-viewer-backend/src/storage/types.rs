use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub basename: String,
    pub lastmod: String,
    pub size: String, // 使用字符串表示大数字
    #[serde(rename = "type")]
    pub file_type: String, // "file" or "directory"
    pub mime: Option<String>,
    pub etag: Option<String>,
}

/// 目录列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryResult {
    pub files: Vec<FileInfo>,
    pub has_more: bool,
    pub next_marker: Option<String>,
    pub total_count: Option<String>,
    pub path: String,
}

/// 列表选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOptions {
    pub page_size: Option<u32>,
    pub marker: Option<String>,
    pub prefix: Option<String>,
    pub recursive: Option<bool>,
    pub sort_by: Option<String>,    // "name", "size", "modified"
    pub sort_order: Option<String>, // "asc", "desc"
}

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub protocol: String,
    pub url: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub endpoint: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    // SSH 特定字段
    pub port: Option<u16>,
    pub private_key_path: Option<String>,
    pub passphrase: Option<String>,
    pub root_path: Option<String>,
    // SMB 特定字段
    pub share: Option<String>,
    pub domain: Option<String>,
    pub extra_options: Option<HashMap<String, String>>,
}

/// 存储客户端错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum StorageError {
    #[error("连接失败: {0}")]
    ConnectionFailed(String),

    #[error("认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("请求失败: {0}")]
    RequestFailed(String),

    #[error("文件未找到: {0}")]
    NotFound(String),

    #[error("配置无效: {0}")]
    InvalidConfig(String),

    #[error("协议不支持: {0}")]
    ProtocolNotSupported(String),

    #[error("未连接")]
    NotConnected,

    #[error("IO 错误: {0}")]
    IoError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),
}

/// 文件内容响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub content: Vec<u8>,
    pub size: u64,
    pub mime_type: Option<String>,
    pub encoding: Option<String>,
}

/// 进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub current: u64,
    pub total: u64,
    pub percentage: f64,
    pub speed: Option<u64>, // bytes per second
    pub eta: Option<u64>,   // seconds
}

/// 下载会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSession {
    pub id: String,
    pub file_path: String,
    pub file_size: u64,
    pub progress: ProgressInfo,
    pub status: DownloadStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}