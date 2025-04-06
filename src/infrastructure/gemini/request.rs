use serde::Serialize;

/// Gemini APIのリクエスト部分を表す構造体
#[derive(Serialize, Debug)]
pub struct Part {
    pub text: String
}

/// Gemini APIのリクエストコンテンツを表す構造体
#[derive(Serialize, Debug)]
pub struct Content {
    pub parts: Vec<Part>
}

/// Gemini APIのリクエストを表す構造体
#[derive(Serialize, Debug)]
pub struct GeminiRequest {
    pub contents: Vec<Content>
}

impl GeminiRequest {
    /// 新しいGemini APIリクエストを作成する
    ///
    /// # Arguments
    /// * `text` - リクエストテキスト
    pub fn new(text: String) -> Self {
        Self {
            contents: vec![Content {
                parts: vec![Part { text }],
            }],
        }
    }
}
