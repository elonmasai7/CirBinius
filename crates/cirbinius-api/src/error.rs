use http::StatusCode;
use http_body_util::Full;
use hyper::body::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Upload too large")]
    UploadTooLarge,

    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("Job failed: {0}")]
    JobFailed(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl ApiError {
    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UploadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::JobFailed(_) => StatusCode::OK,
            ApiError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Serde(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_response(&self) -> http::Response<Full<Bytes>> {
        let body = serde_json::json!({ "error": self.to_string() });
        let bytes = serde_json::to_vec(&body).unwrap_or_default();
        http::Response::builder()
            .status(self.status())
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(bytes)))
            .unwrap()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
