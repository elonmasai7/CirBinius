use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("HTTP error: {status} - {body}")]
    HttpError { status: u16, body: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP parse error: {0}")]
    HttpParse(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),
}

pub type SdkResult<T> = Result<T, SdkError>;
