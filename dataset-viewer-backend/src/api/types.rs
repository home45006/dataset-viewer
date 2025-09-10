use serde::{Deserialize, Serialize};
use crate::storage::{ConnectionConfig, DirectoryResult, FileContent, ListOptions};

/// 标准 API 响应格式
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ApiResponse<T> {
    #[serde(rename = "success")]
    Success { data: T },
    #[serde(rename = "error")]
    Error { error: String, message: String },
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse::Success { data }
    }

    pub fn error(error: impl Into<String>, message: impl Into<String>) -> Self {
        ApiResponse::Error {
            error: error.into(),
            message: message.into(),
        }
    }
}

/// 连接请求
#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub config: ConnectionConfig,
}

/// 连接响应
#[derive(Debug, Serialize)]
pub struct ConnectResponse {
    pub session_id: String,
    pub protocol: String,
    pub connected: bool,
}

/// 文件列表请求
#[derive(Debug, Deserialize)]
pub struct ListRequest {
    pub session_id: String,
    pub path: Option<String>,
    pub options: Option<ListOptions>,
}

/// 文件内容请求
#[derive(Debug, Deserialize)]
pub struct FileContentRequest {
    pub session_id: String,
    pub path: String,
    pub start: Option<u64>,
    pub length: Option<u64>,
}

/// 下载请求
#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub session_id: String,
    pub file_path: String,
    pub save_path: Option<String>,
}

/// 下载响应
#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub download_id: String,
    pub file_size: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// 会话信息
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub protocol: String,
    pub connected: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 服务器状态信息
#[derive(Debug, Serialize)]
pub struct ServerStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub active_connections: usize,
    pub active_sessions: usize,
    pub memory_usage: Option<u64>,
}