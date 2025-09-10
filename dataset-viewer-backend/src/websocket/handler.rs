use crate::{state::AppState, websocket::{WebSocketMessage, WebSocketResponse}};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// WebSocket 处理器
pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// 处理单个 WebSocket 连接
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    // 注册连接
    let (connection_id, mut receiver) = state.websocket_manager.register_connection().await;
    info!("WebSocket 连接已建立: {}", connection_id);

    let (mut sender, mut socket_receiver) = socket.split();

    // 启动消息发送任务
    let connection_id_clone = connection_id.clone();
    let websocket_manager_clone = state.websocket_manager.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(response) = receiver.recv().await {
            // 将响应序列化为 JSON
            match serde_json::to_string(&response) {
                Ok(json_str) => {
                    if sender.send(Message::Text(json_str)).await.is_err() {
                        debug!("WebSocket 发送失败，连接可能已关闭: {}", connection_id_clone);
                        break;
                    }
                }
                Err(e) => {
                    error!("序列化 WebSocket 消息失败: {}", e);
                }
            }
        }

        // 清理连接
        websocket_manager_clone.unregister_connection(&connection_id_clone).await;
        debug!("WebSocket 发送任务结束: {}", connection_id_clone);
    });

    // 启动消息接收任务
    let connection_id_clone = connection_id.clone();
    let websocket_manager_clone = state.websocket_manager.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = socket_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = handle_text_message(&text, &connection_id_clone, &websocket_manager_clone).await {
                        warn!("处理 WebSocket 文本消息失败: {}", e);
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("收到二进制消息，暂不支持");
                }
                Ok(Message::Ping(data)) => {
                    // 自动回复 Pong
                    debug!("收到 Ping，回复 Pong");
                }
                Ok(Message::Pong(_)) => {
                    debug!("收到 Pong");
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket 连接关闭: {}", connection_id_clone);
                    break;
                }
                Err(e) => {
                    error!("WebSocket 消息错误: {}", e);
                    break;
                }
            }
        }

        // 清理连接
        websocket_manager_clone.unregister_connection(&connection_id_clone).await;
        debug!("WebSocket 接收任务结束: {}", connection_id_clone);
    });

    // 等待任务完成
    tokio::select! {
        _ = send_task => {
            debug!("WebSocket 发送任务完成");
        }
        _ = receive_task => {
            debug!("WebSocket 接收任务完成");
        }
    }

    info!("WebSocket 连接已断开: {}", connection_id);
}

/// 处理文本消息
async fn handle_text_message(
    text: &str,
    connection_id: &str,
    websocket_manager: &crate::websocket::WebSocketManager,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 尝试解析为 WebSocket 消息
    let message: WebSocketMessage = serde_json::from_str(text)?;

    match message {
        WebSocketMessage::Subscribe { session_id } => {
            websocket_manager.subscribe_to_session(connection_id, &session_id).await;
            debug!("WebSocket {} 订阅会话: {}", connection_id, session_id);
        }
        WebSocketMessage::Unsubscribe { session_id } => {
            websocket_manager.unsubscribe_from_session(connection_id, &session_id).await;
            debug!("WebSocket {} 取消订阅会话: {}", connection_id, session_id);
        }
        WebSocketMessage::Ping => {
            // 回复 Pong
            websocket_manager.broadcast(WebSocketMessage::Pong).await;
        }
        _ => {
            warn!("收到不支持的 WebSocket 消息类型");
        }
    }

    Ok(())
}