use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 统一的错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("存储错误: {0}")]
    Storage(#[from] crate::storage::StorageError),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("认证错误: {0}")]
    Authentication(String),

    #[error("授权错误: {0}")]
    Authorization(String),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("文件未找到: {0}")]
    NotFound(String),

    #[error("内部服务器错误: {0}")]
    Internal(String),

    #[error("网络错误: {0}")]
    Network(String),

    #[error("压缩包错误: {0}")]
    Archive(String),

    #[error("WebSocket 错误: {0}")]
    WebSocket(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP 错误: {0}")]
    Http(#[from] reqwest::Error),
}

/// 标准化的 API 错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl Error {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::Storage(_) => StatusCode::BAD_REQUEST,
            Error::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Authentication(_) => StatusCode::UNAUTHORIZED,
            Error::Authorization(_) => StatusCode::FORBIDDEN,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Network(_) => StatusCode::BAD_GATEWAY,
            Error::Archive(_) => StatusCode::BAD_REQUEST,
            Error::WebSocket(_) => StatusCode::BAD_REQUEST,
            Error::Serialization(_) => StatusCode::BAD_REQUEST,
            Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Http(_) => StatusCode::BAD_GATEWAY,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Error::Storage(_) => "STORAGE_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            Error::Authentication(_) => "AUTHENTICATION_ERROR",
            Error::Authorization(_) => "AUTHORIZATION_ERROR",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Internal(_) => "INTERNAL_ERROR",
            Error::Network(_) => "NETWORK_ERROR",
            Error::Archive(_) => "ARCHIVE_ERROR",
            Error::WebSocket(_) => "WEBSOCKET_ERROR",
            Error::Serialization(_) => "SERIALIZATION_ERROR",
            Error::Io(_) => "IO_ERROR",
            Error::Http(_) => "HTTP_ERROR",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        
        let error_response = ErrorResponse {
            status: "error".to_string(),
            error: self.error_type().to_string(),
            message: self.to_string(),
            details: None,
        };

        (status, Json(error_response)).into_response()
    }
}

/// 便利的 Result 类型
pub type Result<T> = std::result::Result<T, Error>;