use base64::{engine::general_purpose, Engine as _};
use log::{error, warn};
use rss::Channel;
use std::env;
use std::error::Error;

use crate::domain::ai_service::AiServiceError;
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

    /// Gemini APIのURLを取得する
    fn get_gemini_api_url() -> Result<String, RssSummaryError> {
        let url = env::var("GEMINI_API_URL").unwrap_or_else(|_| {
            error!("GEMINI_API_URL is not set");
            String::new()
        });

        if url.is_empty() {
            return Err(RssSummaryError::EnvVarError(
                "GEMINI_API_URL is empty".to_string(),
            ));
        }

        Ok(url)
    }

    /// Gemini APIリクエストを作成する
    fn create_gemini_request(prompt: &str, rss_data: &[RssData]) -> Result<GeminiRequest, RssSummaryError> {
        let rss_data_str = serde_json::to_string(rss_data)?;
        
        Ok(GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: format!("{}{}", prompt, rss_data_str),
                }],
            }],
        })
    }

    /// レスポンスからサマリーを抽出する
    fn extract_summary_from_response(response: &GeminiResponse) -> Result<ArticlesResponse, RssSummaryError> {
        let summary = response
            .candidates
            .iter()
            .filter_map(|candidate| {
                candidate.content.parts.iter().find_map(|part| {
                    // code blockを削除
                    let part_text = part.text.replace("```json", "").replace("```", "");
                    match serde_json::from_str::<ArticlesResponse>(&part_text) {
                        Ok(summary) => Some(summary),
                        Err(e) => {
                            error!("Failed to parse summary: {}", e);
                            None
                        }
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

        // Gemini API URLの取得
        let url = Self::get_gemini_api_url()?;

        // Gemini APIリクエストの作成
        let gemini_request_body = Self::create_gemini_request(&prompt, &rss_data_items)?;

        // Gemini APIへのリクエスト
        let response: GeminiResponse = self
            .http_client
            .post_with_response(&url, &gemini_request_body)
            .await
            .map_err(|e| RssSummaryError::HttpError(e.to_string()))?;

        // レスポンスからサマリーを抽出
        Self::extract_summary_from_response(&response)
    }
}
