use crate::websocket::{WebSocketMessage, WebSocketResponse};
use axum::extract::ws::{Message, WebSocket};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// WebSocket 连接信息
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub subscribed_sessions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// WebSocket 管理器
pub struct WebSocketManager {
    // 使用 broadcast channel 来广播消息给所有连接
    sender: broadcast::Sender<WebSocketResponse>,
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        
        Self {
            sender,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册新的 WebSocket 连接
    pub async fn register_connection(&self) -> (String, broadcast::Receiver<WebSocketResponse>) {
        let connection_id = Uuid::new_v4().to_string();
        let receiver = self.sender.subscribe();

        let connection = WebSocketConnection {
            id: connection_id.clone(),
            subscribed_sessions: Vec::new(),
            created_at: chrono::Utc::now(),
        };

        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), connection);
        }

        (connection_id, receiver)
    }

    /// 注销 WebSocket 连接
    pub async fn unregister_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
    }

    /// 订阅会话
    pub async fn subscribe_to_session(&self, connection_id: &str, session_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            if !connection.subscribed_sessions.contains(&session_id.to_string()) {
                connection.subscribed_sessions.push(session_id.to_string());
            }
        }
    }

    /// 取消订阅会话
    pub async fn unsubscribe_from_session(&self, connection_id: &str, session_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.subscribed_sessions.retain(|s| s != session_id);
        }
    }

    /// 广播消息
    pub async fn broadcast(&self, message: WebSocketMessage) {
        let response = WebSocketResponse {
            id: Some(Uuid::new_v4().to_string()),
            message,
            timestamp: chrono::Utc::now(),
        };

        // 忽略发送失败（可能没有订阅者）
        let _ = self.sender.send(response);
    }

    /// 发送进度更新
    pub async fn send_progress(
        &self,
        session_id: &str,
        file_path: &str,
        progress: crate::storage::ProgressInfo,
    ) {
        self.broadcast(WebSocketMessage::Progress {
            session_id: session_id.to_string(),
            file_path: file_path.to_string(),
            progress,
        }).await;
    }

    /// 发送下载完成消息
    pub async fn send_download_complete(
        &self,
        session_id: &str,
        file_path: &str,
        success: bool,
        message: Option<String>,
    ) {
        self.broadcast(WebSocketMessage::DownloadComplete {
            session_id: session_id.to_string(),
            file_path: file_path.to_string(),
            success,
            message,
        }).await;
    }

    /// 发送连接状态变化
    pub async fn send_connection_status(
        &self,
        session_id: &str,
        connected: bool,
        protocol: &str,
    ) {
        self.broadcast(WebSocketMessage::ConnectionStatus {
            session_id: session_id.to_string(),
            connected,
            protocol: protocol.to_string(),
        }).await;
    }

    /// 发送错误消息
    pub async fn send_error(
        &self,
        session_id: Option<&str>,
        error: &str,
        details: Option<&str>,
    ) {
        self.broadcast(WebSocketMessage::Error {
            session_id: session_id.map(|s| s.to_string()),
            error: error.to_string(),
            details: details.map(|d| d.to_string()),
        }).await;
    }

    /// 获取连接数量
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// 清理过期连接（可选的维护功能）
    pub async fn cleanup_expired_connections(&self, max_age_hours: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut connections = self.connections.write().await;
        
        connections.retain(|_, conn| conn.created_at > cutoff);
    }
}