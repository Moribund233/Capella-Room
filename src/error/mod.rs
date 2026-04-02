use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("认证失败: {0}")]
    Auth(String),

    #[error("验证失败: {0}")]
    Validation(String),

    #[error("WebSocket错误: {0}")]
    WebSocket(String),

    #[error("未找到资源")]
    NotFound,

    #[error("资源已存在: {0}")]
    Conflict(String),

    #[error("权限不足")]
    Forbidden,

    #[error("请求超时")]
    Timeout,

    #[error("配置错误: {0}")]
    Config(String),

    #[error("内部服务器错误")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 记录错误日志
        error!("Application error: {}", self);

        let (status, error_code, error_message) = match &self {
            AppError::Database(e) => {
                error!("Database error details: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "数据库操作失败",
                )
            }
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", "认证失败"),
            AppError::Validation(_) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", "请求参数错误")
            }
            AppError::WebSocket(_) => {
                (StatusCode::BAD_REQUEST, "WEBSOCKET_ERROR", "WebSocket错误")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "资源未找到"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", "资源已存在"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "权限不足"),
            AppError::Timeout => (StatusCode::REQUEST_TIMEOUT, "TIMEOUT", "请求超时"),
            AppError::Config(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
                "配置错误",
            ),
            AppError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "服务器内部错误",
            ),
        };

        let body = Json(json!({
            "success": false,
            "code": error_code,
            "error": error_message,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        error!("Anyhow error converted: {:?}", error);
        AppError::Internal
    }
}

impl From<std::env::VarError> for AppError {
    fn from(error: std::env::VarError) -> Self {
        AppError::Config(format!("环境变量错误: {}", error))
    }
}

impl From<config::ConfigError> for AppError {
    fn from(error: config::ConfigError) -> Self {
        AppError::Config(format!("配置加载错误: {}", error))
    }
}

/// 应用结果类型
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Auth("Invalid token".to_string());
        assert_eq!(err.to_string(), "认证失败: Invalid token");
    }

    #[test]
    fn test_error_from_sqlx() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err: AppError = sqlx_err.into();
        matches!(app_err, AppError::Database(_));
    }
}
