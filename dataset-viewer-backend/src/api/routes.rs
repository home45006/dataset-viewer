use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::{api::handlers, state::AppState};
use std::sync::Arc;

/// 创建 API 路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 健康检查和状态
        .route("/health", get(handlers::health))
        .route("/status", get(handlers::server_status))

        // 存储连接管理
        .route("/storage/connect", post(handlers::storage::connect))
        .route("/storage/disconnect/:session_id", delete(handlers::storage::disconnect))
        .route("/storage/sessions", get(handlers::storage::list_sessions))
        .route("/storage/sessions/:session_id", get(handlers::storage::get_session))

        // 文件系统操作
        .route("/storage/:session_id/list", post(handlers::storage::list_directory))
        .route("/storage/:session_id/file/content", post(handlers::storage::get_file_content))
        .route("/storage/:session_id/file/info", post(handlers::storage::get_file_info))
        .route("/storage/:session_id/file/download", post(handlers::storage::download_file))

        // 压缩包处理
        .route("/storage/:session_id/archive/info", post(handlers::archive::get_archive_info))
        .route("/storage/:session_id/archive/file", post(handlers::archive::get_archive_file))

        // 文档和版本信息
        .route("/docs", get(handlers::api_docs))
        .route("/version", get(handlers::version))
}