use std::env;

#[derive(Debug, PartialEq, Eq)]
pub enum GeminiConfigError {
    MissingApiKey,
}

impl std::fmt::Display for GeminiConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeminiConfigError::MissingApiKey => write!(f, "GEMINI_API_KEY is not set"),
        }
    }
}

impl std::error::Error for GeminiConfigError {}

#[derive(Debug, PartialEq, Eq)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

impl GeminiConfig {
    pub fn from_env() -> Result<Self, GeminiConfigError> {
        let api_key = env::var("GEMINI_API_KEY").ok().filter(|s| !s.is_empty());
        let model = env::var("GEMINI_MODEL").ok().filter(|s| !s.is_empty());
        Self::from_values(api_key, model)
    }

    pub fn from_values(api_key: Option<String>, model: Option<String>) -> Result<Self, GeminiConfigError> {
        let api_key = api_key.filter(|s| !s.is_empty()).ok_or(GeminiConfigError::MissingApiKey)?;
        let model = model.filter(|s| !s.is_empty()).unwrap_or_else(|| "gemini-3.8-flash".to_string());
        
        Ok(Self { api_key, model })
    }

    pub fn endpoint_url(&self) -> String {
        format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t1_from_values_with_key_and_model() {
        // Given
        let api_key = Some("test-api-key".to_string());
        let model = Some("gemini-3.8-flash".to_string());

        // When
        let result = GeminiConfig::from_values(api_key, model);

        // Then
        assert_eq!(
            result,
            Ok(GeminiConfig {
                api_key: "test-api-key".to_string(),
                model: "gemini-3.8-flash".to_string(),
            })
        );
    }

    #[test]
    fn test_t2_from_values_default_model_when_none() {
        // Given
        let api_key = Some("test-api-key".to_string());
        let model = None;

        // When
        let result = GeminiConfig::from_values(api_key, model);

        // Then
        assert_eq!(result.unwrap().model, "gemini-3.8-flash");
    }

    #[test]
    fn test_t3_from_values_default_model_when_empty() {
        // Given
        let api_key = Some("test-api-key".to_string());
        let model = Some("".to_string());

        // When
        let result = GeminiConfig::from_values(api_key, model);

        // Then
        assert_eq!(result.unwrap().model, "gemini-3.8-flash");
    }

    #[test]
    fn test_t4_from_values_missing_api_key() {
        // Given
        let api_key = None;
        let model = Some("gemini-3.8-flash".to_string());

        // When
        let result = GeminiConfig::from_values(api_key, model);

        // Then
        assert_eq!(result, Err(GeminiConfigError::MissingApiKey));
        assert_eq!(GeminiConfigError::MissingApiKey.to_string(), "GEMINI_API_KEY is not set");
    }

    #[test]
    fn test_t5_from_values_empty_api_key() {
        // Given
        let api_key = Some("".to_string());
        let model = None;

        // When
        let result = GeminiConfig::from_values(api_key, model);

        // Then
        assert_eq!(result, Err(GeminiConfigError::MissingApiKey));
    }

    #[test]
    fn test_t6_endpoint_url_default_model() {
        // Given
        let config = GeminiConfig {
            api_key: "test-api-key".to_string(),
            model: "gemini-3.8-flash".to_string(),
        };

        // When
        let url = config.endpoint_url();

        // Then
        assert_eq!(url, "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.8-flash:generateContent");
    }

    #[test]
    fn test_t7_endpoint_url_custom_model() {
        // Given
        let config = GeminiConfig {
            api_key: "test-api-key".to_string(),
            model: "gemini-3.6-flash".to_string(),
        };

        // When
        let url = config.endpoint_url();

        // Then
        assert_eq!(url, "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.6-flash:generateContent");
    }

    #[test]
    fn test_p1_model_is_never_empty() {
        // Given
        let inputs = vec![
            Some("custom-model".to_string()),
            Some("".to_string()),
            None,
        ];

        // When / Then
        for model_input in inputs {
            let result = GeminiConfig::from_values(Some("key".to_string()), model_input);
            if let Ok(config) = result {
                assert!(!config.model.is_empty(), "Model should never be empty");
            }
        }
    }

    #[test]
    fn test_p2_endpoint_url_format() {
        // Given
        let models = vec![
            "gemini-3.8-flash",
            "gemini-pro",
            "custom-model-123",
        ];

        // When / Then
        for model in models {
            let config = GeminiConfig {
                api_key: "key".to_string(),
                model: model.to_string(),
            };
            let url = config.endpoint_url();
            
            assert!(url.starts_with("https://generativelanguage.googleapis.com/v1beta/models/"), "URL must start with correct prefix");
            assert!(url.ends_with(":generateContent"), "URL must end with correct suffix");
            assert!(url.contains(model), "URL must contain the model name");
        }
    }
}
