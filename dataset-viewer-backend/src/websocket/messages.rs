use serde::{Deserialize, Serialize};
use crate::storage::ProgressInfo;

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketMessage {
    /// 进度更新消息
    Progress {
        session_id: String,
        file_path: String,
        progress: ProgressInfo,
    },
    
    /// 下载完成消息
    DownloadComplete {
        session_id: String,
        file_path: String,
        success: bool,
        message: Option<String>,
    },
    
    /// 连接状态变化
    ConnectionStatus {
        session_id: String,
        connected: bool,
        protocol: String,
    },
    
    /// 错误消息
    Error {
        session_id: Option<String>,
        error: String,
        details: Option<String>,
    },
    
    /// 心跳消息
    Ping,
    Pong,
    
    /// 客户端订阅会话
    Subscribe {
        session_id: String,
    },
    
    /// 客户端取消订阅会话
    Unsubscribe {
        session_id: String,
    },
}

/// WebSocket 响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketResponse {
    pub id: Option<String>,
    pub message: WebSocketMessage,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}