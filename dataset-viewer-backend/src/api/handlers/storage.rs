use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

use crate::{
    api::types::{
        ApiResponse, ConnectRequest, ConnectResponse, FileContentRequest, ListRequest,
        DownloadRequest, DownloadResponse, SessionInfo,
    },
    state::AppState,
    Error,
};

/// 连接到存储服务
pub async fn connect(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ConnectRequest>,
) -> Result<Json<ApiResponse<ConnectResponse>>, Error> {
    let session_id = state.storage_manager.create_client(&request.config).await
        .map_err(|e| Error::Storage(e))?;

    let response = ConnectResponse {
        session_id: session_id.clone(),
        protocol: request.config.protocol.clone(),
        connected: true,
    };

    // 发送连接状态通知
    state
        .websocket_manager
        .send_connection_status(&session_id, true, &request.config.protocol)
        .await;

    Ok(Json(ApiResponse::success(response)))
}

/// 断开存储连接
pub async fn disconnect(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, Error> {
    state.storage_manager.disconnect(&session_id).await
        .map_err(|e| Error::Storage(e))?;

    // 发送连接状态通知
    state
        .websocket_manager
        .send_connection_status(&session_id, false, "unknown")
        .await;

    Ok(Json(ApiResponse::success("Disconnected".to_string())))
}

/// 列出所有会话
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<String>>>, Error> {
    let sessions = state.storage_manager.list_sessions().await;
    Ok(Json(ApiResponse::success(sessions)))
}

/// 获取会话信息
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<ApiResponse<SessionInfo>>, Error> {
    if !state.storage_manager.session_exists(&session_id).await {
        return Err(Error::NotFound("Session not found".to_string()));
    }

    // TODO: 实现会话详细信息获取
    let session_info = SessionInfo {
        session_id,
        protocol: "unknown".to_string(),
        connected: true,
        created_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(session_info)))
}

/// 列出目录内容
pub async fn list_directory(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<ListRequest>,
) -> Result<Json<ApiResponse<crate::storage::DirectoryResult>>, Error> {
    // TODO: 实现目录列表逻辑
    // 由于当前存储管理器设计的限制，这里先返回错误
    Err(Error::Internal("Directory listing not yet implemented".to_string()))
}

/// 获取文件内容
pub async fn get_file_content(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<FileContentRequest>,
) -> Result<Json<ApiResponse<crate::storage::FileContent>>, Error> {
    // TODO: 实现文件内容获取逻辑
    Err(Error::Internal("File content reading not yet implemented".to_string()))
}

/// 获取文件信息
pub async fn get_file_info(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, Error> {
    // TODO: 实现文件信息获取逻辑
    Err(Error::Internal("File info not yet implemented".to_string()))
}

/// 下载文件
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<DownloadRequest>,
) -> Result<Json<ApiResponse<DownloadResponse>>, Error> {
    // TODO: 实现文件下载逻辑
    let download_response = DownloadResponse {
        download_id: uuid::Uuid::new_v4().to_string(),
        file_size: 0,
        started_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(download_response)))
}