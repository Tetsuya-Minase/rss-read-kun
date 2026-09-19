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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    pub thinking_level: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub response_mime_type: String,
    pub response_json_schema: Option<serde_json::Value>,
    pub thinking_config: ThinkingConfig,
}

/// Gemini APIのリクエストを表す構造体
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    pub generation_config: GenerationConfig,
}

impl GeminiRequest {
    /// 新しいGemini APIリクエストを作成する
    ///
    /// # Arguments
    /// * `text` - リクエストテキスト
    pub fn new(text: String) -> Self {
        use crate::infrastructure::gemini::schema::articles_response_schema;
        
        Self {
            contents: vec![Content {
                parts: vec![Part { text }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
                response_json_schema: Some(articles_response_schema()),
                thinking_config: ThinkingConfig {
                    thinking_level: "low".to_string(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_t8_serialize_contents() {
        let req = GeminiRequest::new("要約して[{\"title\":\"a\"}]".to_string());
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value["contents"][0]["parts"][0]["text"], "要約して[{\"title\":\"a\"}]");
    }

    #[test]
    fn test_t9_serialize_thinking_level() {
        let req = GeminiRequest::new("dummy".to_string());
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value["generationConfig"]["thinkingConfig"]["thinkingLevel"], "low");
    }

    #[test]
    fn test_t10_serialize_mime_type() {
        let req = GeminiRequest::new("dummy".to_string());
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value["generationConfig"]["responseMimeType"], "application/json");
    }

    #[test]
    fn test_t11_serialize_json_schema() {
        let req = GeminiRequest::new("dummy".to_string());
        let value = serde_json::to_value(&req).unwrap();
        assert_eq!(value["generationConfig"]["responseJsonSchema"]["type"], "object");
        
        let required = value["generationConfig"]["responseJsonSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&Value::String("message".to_string())));
        assert!(required.contains(&Value::String("data".to_string())));
    }

    #[test]
    fn test_t12_no_old_parameters() {
        let req = GeminiRequest::new("dummy".to_string());
        let value = serde_json::to_value(&req).unwrap();
        let gen_config = &value["generationConfig"];
        assert!(gen_config.get("temperature").is_none());
        assert!(gen_config.get("topP").is_none());
        assert!(gen_config.get("topK").is_none());
        assert!(gen_config.get("thinkingBudget").is_none());
    }

    #[test]
    fn test_p3_no_api_key_in_request() {
        let req = GeminiRequest::new("dummy".to_string());
        let json_str = serde_json::to_string(&req).unwrap();
        let api_key = "secret_api_key_12345";
        assert!(!json_str.contains(api_key));
    }
}
