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
    pub content: Option<Content>,
    pub finish_reason: Option<String>,
    pub avg_logprobs: Option<f64>,
}

/// Gemini APIのレスポンスコンテンツを表す構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: Option<String>,
}

/// Gemini APIのレスポンス部分を表す構造体
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub text: Option<String>,
    pub thought: Option<bool>,
    pub thought_signature: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t14_deserialize_full_response() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"{\"message\":\"ok\"}"}],"role":"model"},"finishReason":"STOP","avgLogprobs":-0.5}]}"#;
        let res: Result<GeminiResponse, _> = serde_json::from_str(json);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.candidates[0].content.as_ref().unwrap().parts[0].text.as_deref(), Some("{\"message\":\"ok\"}"));
    }

    #[test]
    fn test_t15_deserialize_missing_avg_logprobs() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"ok"}],"role":"model"},"finishReason":"STOP"}]}"#;
        let res: Result<GeminiResponse, _> = serde_json::from_str(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().candidates[0].avg_logprobs, None);
    }

    #[test]
    fn test_t16_deserialize_missing_finish_reason() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"ok"}],"role":"model"}}]}"#;
        let res: Result<GeminiResponse, _> = serde_json::from_str(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().candidates[0].finish_reason, None);
    }

    #[test]
    fn test_t17_deserialize_missing_role() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"ok"}]}}]}"#;
        let res: Result<GeminiResponse, _> = serde_json::from_str(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().candidates[0].content.as_ref().unwrap().role, None);
    }

    #[test]
    fn test_t18_deserialize_thought_parts() {
        let json = r#"{"candidates":[{"content":{"parts":[{"thought":true,"thoughtSignature":"AbC123"},{"text":"ok"}],"role":"model"}}]}"#;
        let res: Result<GeminiResponse, _> = serde_json::from_str(json);
        assert!(res.is_ok());
        let res = res.unwrap();
        let parts = &res.candidates[0].content.as_ref().unwrap().parts;
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].text, None);
        assert_eq!(parts[0].thought, Some(true));
    }

    #[test]
    fn test_t19_deserialize_empty_candidates() {
        let json = r#"{"candidates":[]}"#;
        let res: Result<GeminiResponse, _> = serde_json::from_str(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().candidates.len(), 0);
    }

    #[test]
    fn test_p4_deserialize_optional_fields() {
        let inputs = vec![
            r#"{"candidates":[{"content":null,"finishReason":null,"avgLogprobs":null}]}"#,
            r#"{"candidates":[{}]}"#,
        ];
        for json in inputs {
            let res: Result<GeminiResponse, _> = serde_json::from_str(json);
            assert!(res.is_ok(), "Failed to parse: {}", json);
        }
    }
}
