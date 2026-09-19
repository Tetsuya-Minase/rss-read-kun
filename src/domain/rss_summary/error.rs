use std::error::Error;
use std::fmt;

/// RSSサマリー関連のエラー型
#[derive(Debug)]
pub enum RssSummaryError {
    HttpError(String),
    EnvVarError(String),
    JsonError(String),
    SummaryError(String),
}

impl fmt::Display for RssSummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RssSummaryError::HttpError(e) => write!(f, "HTTP error: {}", e),
            RssSummaryError::EnvVarError(e) => write!(f, "Environment variable error: {}", e),
            RssSummaryError::JsonError(e) => write!(f, "JSON error: {}", e),
            RssSummaryError::SummaryError(e) => write!(f, "Summary error: {}", e),
        }
    }
}

impl Error for RssSummaryError {}

impl From<std::env::VarError> for RssSummaryError {
    fn from(err: std::env::VarError) -> Self {
        RssSummaryError::EnvVarError(err.to_string())
    }
}

impl From<serde_json::Error> for RssSummaryError {
    fn from(err: serde_json::Error) -> Self {
        RssSummaryError::JsonError(err.to_string())
    }
}

impl<E: Error + 'static> From<Box<E>> for RssSummaryError {
    fn from(err: Box<E>) -> Self {
        RssSummaryError::SummaryError(err.to_string())
    }
}
