use crate::domain::ai_service::{AiRequest, AiResponse, AiService, AiServiceError};
use crate::infrastructure::gemini::request::GeminiRequest;
use crate::infrastructure::gemini::response::GeminiResponse;
use crate::infrastructure::http_client::HttpClient;
use serde_json::Value;

/// Gemini AI サービスの実装
pub struct GeminiAiService<T: HttpClient> {
    http_client: T,
    api_url: String,
}

impl<T: HttpClient> GeminiAiService<T> {
    /// 新しい Gemini AI サービスを作成する
    ///
    /// # Arguments
    /// * `http_client` - HTTP クライアント
    /// * `api_url` - Gemini API の URL
    pub fn new(http_client: T, api_url: String) -> Self {
        Self {
            http_client,
            api_url,
        }
    }

    /// レスポンスからテキストを抽出する
    ///
    /// # Arguments
    /// * `response` - Gemini API のレスポンス
    fn extract_text_from_response(&self, response: &GeminiResponse) -> Result<String, AiServiceError> {
        let text = response
            .candidates
            .iter()
            .filter_map(|candidate| {
                candidate.content.parts.iter().find_map(|part| {
                    if !part.text.is_empty() {
                        Some(part.text.clone())
                    } else {
                        None
                    }
                })
            })
            .next();

        text.ok_or_else(|| {
            AiServiceError::ResponseError("Failed to extract text from response".to_string())
        })
    }

    /// JSON レスポンスを解析する
    ///
    /// # Arguments
    /// * `text` - JSON 形式のテキスト
    fn parse_json_response(&self, text: &str) -> Result<Value, AiServiceError> {
        // code block を削除
        let cleaned_text = text.replace("```json", "").replace("```", "");
        
        serde_json::from_str(&cleaned_text).map_err(|e| {
            AiServiceError::ParseError(format!("Failed to parse JSON response: {}", e))
        })
    }
}

impl<T: HttpClient + Send + Sync + 'static> AiService for GeminiAiService<T> {
    async fn process_request(&self, request: AiRequest) -> Result<AiResponse, AiServiceError> {
        // Gemini リクエストの作成
        let prompt_with_context = format!("{}\n{}", request.prompt, request.context);
        let gemini_request = GeminiRequest::new(prompt_with_context);

        // Gemini API へのリクエスト
        let response: GeminiResponse = self
            .http_client
            .post_with_response(&self.api_url, &gemini_request)
            .await
            .map_err(|e| AiServiceError::RequestError(e.to_string()))?;

        // レスポンスからテキストを抽出
        let content = self.extract_text_from_response(&response)?;

        Ok(AiResponse { content })
    }
}
