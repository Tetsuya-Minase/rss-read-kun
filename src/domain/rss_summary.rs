use std::error::Error;
use std::fmt;

/// RSSサマリー関連のエラー型
#[derive(Debug)]
pub enum RssSummaryError {
    HttpError(String),
    EnvVarError(String),
    JsonError(String),
    Base64Error(String),
    Utf8Error(String),
    SummaryError(String),
}

impl fmt::Display for RssSummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RssSummaryError::HttpError(e) => write!(f, "HTTP error: {}", e),
            RssSummaryError::EnvVarError(e) => write!(f, "Environment variable error: {}", e),
            RssSummaryError::JsonError(e) => write!(f, "JSON error: {}", e),
            RssSummaryError::Base64Error(e) => write!(f, "Base64 error: {}", e),
            RssSummaryError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
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

impl From<base64::DecodeError> for RssSummaryError {
    fn from(err: base64::DecodeError) -> Self {
        RssSummaryError::Base64Error(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for RssSummaryError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        RssSummaryError::Utf8Error(err.to_string())
    }
}

impl<E: Error + 'static> From<Box<E>> for RssSummaryError {
    fn from(err: Box<E>) -> Self {
        RssSummaryError::SummaryError(err.to_string())
    }
}

/// RSSサマリーサービスのトレイト
pub trait RssSummaryService {
    /// RSSフィードから要約を取得する
    ///
    /// # Arguments
    /// * `rss_channel` - RSSチャンネルデータ
    fn fetch_summary<'a>(&'a self, rss_channel: &'a rss::Channel) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<crate::model::rss_summary::ArticlesResponse, RssSummaryError>> + Send + 'a>>;
}
