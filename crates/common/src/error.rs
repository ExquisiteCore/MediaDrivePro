use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未找到: {0}")]
    NotFound(String),

    #[error("未登录")]
    Unauthorized,

    #[error("无权限")]
    Forbidden,

    #[error("参数错误: {0}")]
    Validation(String),

    #[error("资源冲突: {0}")]
    Conflict(String),

    #[error("文件过大")]
    PayloadTooLarge,

    #[error("存储空间不足")]
    QuotaExceeded,

    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("存储错误: {0}")]
    Storage(#[from] opendal::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::PayloadTooLarge | Self::QuotaExceeded => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Database(_) | Self::Storage(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Unauthorized => "AUTH_REQUIRED",
            Self::Forbidden => "FORBIDDEN",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Conflict(_) => "CONFLICT",
            Self::PayloadTooLarge => "PAYLOAD_TOO_LARGE",
            Self::QuotaExceeded => "QUOTA_EXCEEDED",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();

        // Log server errors
        match &self {
            Self::Database(e) => tracing::error!("Database error: {e}"),
            Self::Storage(e) => tracing::error!("Storage error: {e}"),
            Self::Internal(e) => tracing::error!("Internal error: {e}"),
            _ => {}
        }

        let body = json!({
            "error": {
                "code": code,
                "message": self.to_string(),
            }
        });

        (status, axum::Json(body)).into_response()
    }
}
