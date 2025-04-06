use serde::{Deserialize, Serialize};

/// Gemini APIのレスポンスを表す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

/// Gemini APIのレスポンス候補を表す構造体
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: String,
    pub avg_logprobs: f64,
}

/// Gemini APIのレスポンスコンテンツを表す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: String,
}

/// Gemini APIのレスポンス部分を表す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}
