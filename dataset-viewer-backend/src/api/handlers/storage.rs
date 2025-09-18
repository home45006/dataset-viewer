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
    // 检查会话是否存在
    if !state.storage_manager.session_exists(&session_id).await {
        return Err(Error::NotFound("Session not found".to_string()));
    }

    // 构建列表选项
    let options = if let Some(opts) = &request.options {
        crate::storage::ListOptions {
            page_size: opts.page_size,
            marker: opts.marker.clone(),
            prefix: opts.filter.clone(), // 将filter映射到prefix
            recursive: Some(false),
            sort_by: opts.sort_by.clone(),
            sort_order: opts.sort_order.clone(),
        }
    } else {
        crate::storage::ListOptions {
            page_size: None,
            marker: None,
            prefix: None,
            recursive: Some(false),
            sort_by: None,
            sort_order: None,
        }
    };

    // 获取路径，默认为空字符串
    let path = request.path.as_deref().unwrap_or("");

    // 调用存储管理器列出目录内容
    let result = state
        .storage_manager
        .list_directory(&session_id, path, Some(&options))
        .await
        .map_err(|e| Error::Storage(e))?;

    Ok(Json(ApiResponse::success(result)))
}

/// 获取文件内容
pub async fn get_file_content(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<FileContentRequest>,
) -> Result<Json<ApiResponse<crate::storage::FileContent>>, Error> {
    // 检查会话是否存在
    if !state.storage_manager.session_exists(&session_id).await {
        return Err(Error::NotFound("Session not found".to_string()));
    }

    // 获取文件内容
    let result = state
        .storage_manager
        .get_file_content(&session_id, &request.path, request.start, request.length)
        .await
        .map_err(|e| Error::Storage(e))?;

    Ok(Json(ApiResponse::success(result)))
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