use serde_json::{json, Value};

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
                    "summary": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": {
                                "type": "object",
                                "properties": {
                                    "category_count": { "type": "integer" },
                                    "articles": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "title": { "type": "string" },
                                                "description": { "type": "string" },
                                                "link": { "type": "string" }
                                            },
                                            "required": ["title", "description", "link"]
                                        }
                                    }
                                },
                                "required": ["category_count", "articles"]
                            }
                        }
                    }
                },
                "required": ["total", "summary"]
            }
        },
        "required": ["message", "data"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t13_articles_response_schema() {
        let schema = articles_response_schema();
        
        let articles_type = schema["properties"]["data"]["properties"]["summary"]["items"]
            ["additionalProperties"]["properties"]["articles"]["type"]
            .as_str()
            .unwrap();
            
        assert_eq!(articles_type, "array");
    }
}
