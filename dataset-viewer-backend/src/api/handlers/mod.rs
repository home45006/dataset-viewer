pub mod storage;
pub mod archive;

use axum::{response::Html, Json};
use crate::api::types::{ApiResponse, ServerStatus};
use std::sync::Arc;
use crate::state::AppState;

/// 健康检查
pub async fn health() -> &'static str {
    "OK"
}

/// 服务器状态
pub async fn server_status() -> Json<ApiResponse<ServerStatus>> {
    // TODO: 实现真实的服务器状态检查
    let status = ServerStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: 实现运行时间计算
        active_connections: 0, // TODO: 获取活动连接数
        active_sessions: 0, // TODO: 获取活动会话数
        memory_usage: None, // TODO: 获取内存使用情况
    };

    Json(ApiResponse::success(status))
}

/// API 文档
pub async fn api_docs() -> Html<String> {
    let docs = include_str!("../../../docs/api.md").to_string();
    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Dataset Viewer API Documentation</title>
            <meta charset="utf-8">
            <style>
                body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; }}
                pre {{ background: #f5f5f5; padding: 1rem; border-radius: 4px; overflow-x: auto; }}
                code {{ background: #f5f5f5; padding: 0.2rem 0.4rem; border-radius: 2px; }}
            </style>
        </head>
        <body>
            <h1>Dataset Viewer API Documentation</h1>
            <pre>{}</pre>
        </body>
        </html>
        "#,
        docs
    ))
}

/// 版本信息
pub async fn version() -> Json<ApiResponse<serde_json::Value>> {
    let version_info = serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "build_date": env!("BUILD_DATE", "unknown"),
        "git_hash": env!("GIT_HASH", "unknown"),
        "rust_version": env!("RUST_VERSION", "unknown"),
    });

    Json(ApiResponse::success(version_info))
}