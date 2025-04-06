use std::error::Error;
use std::fmt;

/// AI関連のエラー型
#[derive(Debug)]
pub enum AiServiceError {
    RequestError(String),
    ResponseError(String),
    ParseError(String),
}

impl fmt::Display for AiServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiServiceError::RequestError(e) => write!(f, "AI request error: {}", e),
            AiServiceError::ResponseError(e) => write!(f, "AI response error: {}", e),
            AiServiceError::ParseError(e) => write!(f, "AI parse error: {}", e),
        }
    }
}

impl Error for AiServiceError {}

/// AIリクエストの内容を表す構造体
#[derive(Debug)]
pub struct AiRequest {
    pub prompt: String,
    pub context: String,
}

/// AIレスポンスの内容を表す構造体
#[derive(Debug)]
pub struct AiResponse {
    pub content: String,
}

/// AIサービスのトレイト
pub trait AiService {
    /// AIにリクエストを送信し、レスポンスを取得する
    ///
    /// # Arguments
    /// * `request` - AIへのリクエスト
    async fn process_request(&self, request: AiRequest) -> Result<AiResponse, AiServiceError>;
}
