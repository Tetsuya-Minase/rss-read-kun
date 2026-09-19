use serde_json::{json, Value};

pub const CATEGORY_NAMES: [&str; 7] = [
    "🤖 AI・LLM",
    "🌐 Web・フロントエンド",
    "🛠️ バックエンド・インフラ",
    "📱 モバイル・デスクトップ",
    "🎮 ゲーム開発",
    "🧑‍💼 キャリア・組織",
    "📚 その他",
];

/// ArticlesResponse の JSON Schema を返す
pub fn articles_response_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "message": { "type": "string" },
            "data": {
                "type": "object",
                "properties": {
                    "total": { "type": "integer" },
                    "highlights": {
                        "type": "array",
                        "maxItems": 3,
                        "items": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string" },
                                "summary": { "type": "string" },
                                "link": { "type": "string" }
                            },
                            "required": ["title", "summary", "link"]
                        }
                    },
                    "categories": {
                        "type": "array",
                        "maxItems": 7,
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {
                                    "type": "string",
                                    "enum": CATEGORY_NAMES
                                },
                                "articles": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "title": { "type": "string" },
                                            "summary": { "type": "string" },
                                            "link": { "type": "string" }
                                        },
                                        "required": ["title", "summary", "link"]
                                    }
                                }
                            },
                            "required": ["name", "articles"]
                        }
                    }
                },
                "required": ["total", "highlights", "categories"]
            }
        },
        "required": ["message", "data"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_t23_data_required_new_structure() {
        let schema = articles_response_schema();
        let required = schema["properties"]["data"]["required"].as_array().unwrap();
        assert!(required.contains(&Value::String("total".to_string())));
        assert!(required.contains(&Value::String("highlights".to_string())));
        assert!(required.contains(&Value::String("categories".to_string())));
        assert_eq!(required.len(), 3);
    }

    #[test]
    fn test_t24_category_enum_fixed() {
        let schema = articles_response_schema();
        let enum_vals = schema["properties"]["data"]["properties"]["categories"]["items"]["properties"]["name"]["enum"].as_array().unwrap();
        assert_eq!(enum_vals.len(), 7);
        assert!(enum_vals.contains(&Value::String("🤖 AI・LLM".to_string())));
    }

    #[test]
    fn test_t25_highlights_max_items() {
        let schema = articles_response_schema();
        let max_items = schema["properties"]["data"]["properties"]["highlights"]["maxItems"].as_u64().unwrap();
        assert_eq!(max_items, 3);
    }

    #[test]
    fn test_t26_article_required_keys() {
        let schema = articles_response_schema();
        let required = schema["properties"]["data"]["properties"]["categories"]["items"]["properties"]["articles"]["items"]["required"].as_array().unwrap();
        let expected = vec![
            Value::String("title".to_string()),
            Value::String("summary".to_string()),
            Value::String("link".to_string())
        ];
        assert_eq!(required, &expected);
    }

    #[test]
    fn test_t27_no_old_structure_traces() {
        let schema_str = serde_json::to_string(&articles_response_schema()).unwrap();
        assert!(!schema_str.contains("category_count"));
        assert!(!schema_str.contains("additionalProperties"));
    }

    #[test]
    fn test_t56_categories_max_items() {
        let schema = articles_response_schema();
        let max_items = schema["properties"]["data"]["properties"]["categories"]["maxItems"].as_u64().unwrap();
        assert_eq!(max_items, 7);
    }

    #[test]
    fn test_t57_no_unsupported_json_schema_keywords() {
        let schema_str = serde_json::to_string(&articles_response_schema()).unwrap();
        assert!(!schema_str.contains("maxLength"));
        assert!(!schema_str.contains("minLength"));
        assert!(!schema_str.contains("pattern"));
    }
}
