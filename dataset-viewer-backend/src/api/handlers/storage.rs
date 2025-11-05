use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::{
    api::types::{
        ApiResponse, ConnectRequest, ConnectResponse, FileContentRequest, ListRequest,
        DownloadRequest, SessionInfo,
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
) -> Result<impl IntoResponse, Error> {
    // 检查会话是否存在
    if !state.storage_manager.session_exists(&session_id).await {
        return Err(Error::NotFound("Session not found".to_string()));
    }

    // 提前克隆文件路径，避免生命周期问题
    let file_path = request.file_path.clone();

    // 获取文件大小
    let file_size = state
        .storage_manager
        .get_file_size(&session_id, &file_path)
        .await
        .map_err(|e| Error::Storage(e))?;

    // 获取文件名（从路径中提取）
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    // 创建一个流式响应
    // 我们将使用分块方式读取文件内容
    const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB chunks

    // 创建一个异步流来读取文件
    let stream = async_stream::stream! {
        let mut offset = 0u64;

        while offset < file_size {
            let chunk_size = std::cmp::min(CHUNK_SIZE, file_size - offset);

            match state
                .storage_manager
                .read_file_range(&session_id, &file_path, offset, chunk_size)
                .await
            {
                Ok(chunk) => {
                    yield Ok::<_, std::io::Error>(chunk);
                    offset += chunk_size;
                }
                Err(e) => {
                    eprintln!("读取文件分块失败: {}", e);
                    yield Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string()
                    ));
                    break;
                }
            }
        }
    };

    // 将流转换为Body
    let body = Body::from_stream(stream);

    // 构建响应
    use axum::response::Response;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .body(body)
        .map_err(|e| Error::Internal(format!("构建响应失败: {}", e)))?;

    Ok(response)
}