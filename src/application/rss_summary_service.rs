use base64::{engine::general_purpose, Engine as _};
use log::{error, warn};
use rss::Channel;
use std::env;
use std::error::Error;


use crate::domain::model::rss_data::RssData;
use crate::domain::rss_summary::{RssSummaryError, RssSummaryService};
use crate::domain::model::rss_summary::ArticlesResponse;
use crate::infrastructure::gemini::{GeminiRequest, GeminiResponse};
use crate::infrastructure::gemini::request::{Content, Part};
use crate::infrastructure::http_client::HttpClient;

/// RSSサマリーサービスの実装
pub struct RssSummaryServiceImpl<T: HttpClient> {
    http_client: T,
}

impl<T: HttpClient> RssSummaryServiceImpl<T> {
    /// 新しいRSSサマリーサービスを作成する
    ///
    /// # Arguments
    /// * `http_client` - HTTPクライアント
    pub fn new(http_client: T) -> Self {
        Self { http_client }
    }

    /// Base64エンコードされた設定を取得する
    fn get_decoded_config() -> Result<Option<String>, Box<dyn Error>> {
        // 環境変数が存在しない場合は None を返す
        let encoded_config = match env::var("SUMMARY_PROMPT") {
            Ok(val) => val,
            Err(_) => return Ok(None),
        };

        // デコード処理
        let decoded = general_purpose::STANDARD.decode(encoded_config)?;
        let config_str = String::from_utf8(decoded)?;

        Ok(Some(config_str))
    }

    /// RSSデータをモデルに変換する
    fn convert_to_rss_data(rss_channel: &Channel) -> Vec<RssData> {
        rss_channel
            .items
            .iter()
            .map(|item| RssData {
                title: item.title.as_ref().cloned(),
                description: item.description.as_ref().cloned(),
                link: item.link.as_ref().cloned(),
            })
            .collect()
    }


    /// Gemini APIリクエストを作成する
    fn create_gemini_request(prompt: &str, rss_data: &[RssData]) -> Result<GeminiRequest, RssSummaryError> {
        let rss_data_str = serde_json::to_string(rss_data)?;
        
        Ok(GeminiRequest::new(format!("{}{}", prompt, rss_data_str)))
    }

    /// レスポンスからサマリーを抽出する
    fn extract_summary_from_response(response: &GeminiResponse) -> Result<ArticlesResponse, RssSummaryError> {
        let summary = response
            .candidates
            .iter()
            .filter_map(|candidate| {
                let content = candidate.content.as_ref()?;
                content.parts.iter().find_map(|part| {
                    if part.thought == Some(true) {
                        return None;
                    }
                    if let Some(text) = &part.text {
                        // code blockを削除
                        let part_text = text.replace("```json", "").replace("```", "");
                        match serde_json::from_str::<ArticlesResponse>(&part_text) {
                            Ok(summary) => Some(summary),
                            Err(e) => {
                                error!("Failed to parse summary: {}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
            })
            .next();

        summary.ok_or_else(|| {
            RssSummaryError::SummaryError("Failed to extract summary from response".to_string())
        })
    }
}

impl<T: HttpClient + Send + Sync + 'static> RssSummaryService for RssSummaryServiceImpl<T> {
    async fn fetch_summary(&self, rss_channel: &Channel) -> Result<ArticlesResponse, RssSummaryError> {
        // RSSデータをモデルに変換
        let rss_data_items = Self::convert_to_rss_data(rss_channel);

        // プロンプトの取得
        let prompt = match Self::get_decoded_config() {
            Ok(Some(p)) => p,
            Ok(None) => {
                warn!("No summary prompt found, using empty prompt");
                return Err(RssSummaryError::SummaryError("using empty prompt.".to_string()));
            }
            Err(e) => {
                error!("Error decoding config: {}", e);
                return Err(RssSummaryError::SummaryError(e.to_string()));
            }
        };

        // Gemini設定の取得
        use crate::infrastructure::gemini::config::GeminiConfig;
        let config = GeminiConfig::from_env()
            .map_err(|e| RssSummaryError::EnvVarError(e.to_string()))?;

        // Gemini API URLの取得
        let url = config.endpoint_url();

        // Gemini APIリクエストの作成
        let gemini_request_body = Self::create_gemini_request(&prompt, &rss_data_items)?;

        // Gemini APIへのリクエスト
        let headers = vec![("x-goog-api-key".to_string(), config.api_key)];
        let response: GeminiResponse = self
            .http_client
            .post_with_response_and_headers(&url, headers, &gemini_request_body)
            .await
            .map_err(|e| RssSummaryError::HttpError(e.to_string()))?;

        // レスポンスからサマリーを抽出
        Self::extract_summary_from_response(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::gemini::response::{Candidate, Content, Part};
    use crate::infrastructure::http_client::{HttpClient, HttpClientError};
    use rss::Channel;

    struct MockHttpClient;
    impl HttpClient for MockHttpClient {
        fn get(&self, _url: &str) -> impl std::future::Future<Output = Result<Channel, HttpClientError>> + Send { async { unimplemented!() } }
        fn post<T: serde::Serialize + ?Sized + Send + Sync>(&self, _url: &str, _body: &T) -> impl std::future::Future<Output = Result<(), HttpClientError>> + Send { async { unimplemented!() } }
        fn post_with_response<T: serde::Serialize + ?Sized + Send + Sync, R: for<'de> serde::Deserialize<'de> + Send>(&self, _url: &str, _body: &T) -> impl std::future::Future<Output = Result<R, HttpClientError>> + Send { async { unimplemented!() } }
        fn post_with_response_and_headers<T: serde::Serialize + ?Sized + Send + Sync, R: for<'de> serde::Deserialize<'de> + Send>(&self, _url: &str, _headers: Vec<(String, String)>, _body: &T) -> impl std::future::Future<Output = Result<R, HttpClientError>> + Send { async { unimplemented!() } }
    }

    #[test]
    fn test_t20_extract_summary_normal_json() {
        let response = GeminiResponse {
            candidates: vec![Candidate {
                content: Some(Content {
                    parts: vec![Part {
                        text: Some(r#"{"message":"Success","data":{"total":1,"summary":[{"Tech":{"category_count":1,"articles":[{"title":"t","description":"d","link":"https://example.com/a"}]}}]}}"#.to_string()),
                        thought: None,
                        thought_signature: None,
                    }],
                    role: Some("model".to_string()),
                }),
                finish_reason: Some("STOP".to_string()),
                avg_logprobs: None,
            }],
        };
        let res = RssSummaryServiceImpl::<MockHttpClient>::extract_summary_from_response(&response);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.message, "Success");
        assert_eq!(res.data.total, 1);
        assert_eq!(res.data.summary[0].get_name(), Some("Tech".to_string()));
    }

    #[test]
    fn test_t21_extract_summary_with_code_fence() {
        let response = GeminiResponse {
            candidates: vec![Candidate {
                content: Some(Content {
                    parts: vec![Part {
                        text: Some("```json\n{\"message\":\"Success\",\"data\":{\"total\":0,\"summary\":[]}}\n```".to_string()),
                        thought: None,
                        thought_signature: None,
                    }],
                    role: None,
                }),
                finish_reason: None,
                avg_logprobs: None,
            }],
        };
        let res = RssSummaryServiceImpl::<MockHttpClient>::extract_summary_from_response(&response);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().message, "Success");
    }

    #[test]
    fn test_t22_extract_summary_ignores_thought() {
        let response = GeminiResponse {
            candidates: vec![Candidate {
                content: Some(Content {
                    parts: vec![
                        Part {
                            text: Some("考え中: JSONを組み立てる".to_string()),
                            thought: Some(true),
                            thought_signature: None,
                        },
                        Part {
                            text: Some(r#"{"message":"Success","data":{"total":0,"summary":[]}}"#.to_string()),
                            thought: None,
                            thought_signature: None,
                        }
                    ],
                    role: None,
                }),
                finish_reason: None,
                avg_logprobs: None,
            }],
        };
        let res = RssSummaryServiceImpl::<MockHttpClient>::extract_summary_from_response(&response);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().message, "Success");
    }

    #[test]
    fn test_t23_extract_summary_empty_candidates() {
        let response = GeminiResponse { candidates: vec![] };
        let res = RssSummaryServiceImpl::<MockHttpClient>::extract_summary_from_response(&response);
        assert!(res.is_err());
        match res.unwrap_err() {
            RssSummaryError::SummaryError(msg) => assert_eq!(msg, "Failed to extract summary from response"),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_t24_extract_summary_invalid_json() {
        let response = GeminiResponse {
            candidates: vec![Candidate {
                content: Some(Content {
                    parts: vec![Part {
                        text: Some("I cannot answer that.".to_string()),
                        thought: None,
                        thought_signature: None,
                    }],
                    role: None,
                }),
                finish_reason: None,
                avg_logprobs: None,
            }],
        };
        let res = RssSummaryServiceImpl::<MockHttpClient>::extract_summary_from_response(&response);
        assert!(res.is_err());
        match res.unwrap_err() {
            RssSummaryError::SummaryError(msg) => assert_eq!(msg, "Failed to extract summary from response"),
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_p5_thought_text_never_extracted() {
        let response = GeminiResponse {
            candidates: vec![Candidate {
                content: Some(Content {
                    parts: vec![Part {
                        text: Some(r#"{"message":"Danger","data":{"total":0,"summary":[]}}"#.to_string()),
                        thought: Some(true),
                        thought_signature: None,
                    }],
                    role: None,
                }),
                finish_reason: None,
                avg_logprobs: None,
            }],
        };
        let res = RssSummaryServiceImpl::<MockHttpClient>::extract_summary_from_response(&response);
        assert!(res.is_err()); // Should not extract from thought part
    }

    #[actix_web::test]
    async fn test_t25_t26_fetch_summary() {
        use std::sync::{Arc, Mutex};
        
        struct MockHttpClientWithHeaders {
            called: Arc<Mutex<usize>>,
            url: Arc<Mutex<String>>,
            headers: Arc<Mutex<Vec<(String, String)>>>,
        }
        impl HttpClient for MockHttpClientWithHeaders {
            fn get(&self, _url: &str) -> impl std::future::Future<Output = Result<Channel, HttpClientError>> + Send { async { unimplemented!() } }
            fn post<T: serde::Serialize + ?Sized + Send + Sync>(&self, _url: &str, _body: &T) -> impl std::future::Future<Output = Result<(), HttpClientError>> + Send { async { unimplemented!() } }
            fn post_with_response<T: serde::Serialize + ?Sized + Send + Sync, R: for<'de> serde::Deserialize<'de> + Send>(&self, _url: &str, _body: &T) -> impl std::future::Future<Output = Result<R, HttpClientError>> + Send { async { unimplemented!() } }
            fn post_with_response_and_headers<T: serde::Serialize + ?Sized + Send + Sync, R: for<'de> serde::Deserialize<'de> + Send>(&self, url: &str, headers: Vec<(String, String)>, _body: &T) -> impl std::future::Future<Output = Result<R, HttpClientError>> + Send { 
                let called = self.called.clone();
                let url_out = self.url.clone();
                let headers_out = self.headers.clone();
                let url_str = url.to_string();
                async move { 
                    *called.lock().unwrap() += 1;
                    *url_out.lock().unwrap() = url_str;
                    *headers_out.lock().unwrap() = headers;
                    
                    let json = r#"{"candidates":[{"content":{"parts":[{"text":"{\"message\":\"Success\",\"data\":{\"total\":1,\"summary\":[]}}"}]}}]}"#;
                    Ok(serde_json::from_str(json).unwrap())
                } 
            }
        }

        let called = Arc::new(Mutex::new(0));
        let url = Arc::new(Mutex::new(String::new()));
        let headers = Arc::new(Mutex::new(Vec::new()));
        
        let client = MockHttpClientWithHeaders {
            called: called.clone(),
            url: url.clone(),
            headers: headers.clone(),
        };
        let service = RssSummaryServiceImpl::new(client);

        // test_t26: missing API key
        std::env::remove_var("GEMINI_API_KEY");
        std::env::set_var("SUMMARY_PROMPT", "44Kv44OV44K/"); // Some valid base64
        
        let res = service.fetch_summary(&Channel::default()).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            RssSummaryError::EnvVarError(_) => {},
            _ => panic!("Expected EnvVarError"),
        }
        assert_eq!(*called.lock().unwrap(), 0);

        // test_t25: with API key
        std::env::set_var("GEMINI_API_KEY", "test-api-key");
        std::env::remove_var("GEMINI_MODEL");
        let res = service.fetch_summary(&Channel::default()).await;
        assert!(res.is_ok());
        
        assert_eq!(*called.lock().unwrap(), 1);
        let actual_url = url.lock().unwrap().clone();
        assert_eq!(actual_url, "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.8-flash:generateContent");
        assert!(!actual_url.contains("key="));
        
        let actual_headers = headers.lock().unwrap().clone();
        assert!(actual_headers.contains(&("x-goog-api-key".to_string(), "test-api-key".to_string())));
    }

}
