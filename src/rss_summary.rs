use crate::infrastructure::http_client::{HttpClient, HttpClientImpl};
use crate::model::gemini_request::{Content, GeminiRequest, Part};
use crate::model::gemini_response::GeminiResponse;
use crate::model::rss_data::RssData;
use crate::model::rss_summary::ArticlesResponse;
use base64::{engine::general_purpose, Engine as _};
use log::{error, warn};
use rss::Channel;
use std::env;
use std::error::Error;
use std::fmt;

/// RSSサマリー関連のエラー型
#[derive(Debug)]
pub enum RssSummaryError {
    HttpError(String),
    EnvVarError(String),
    JsonError(String),
    Base64Error(String),
    Utf8Error(String),
    SummaryError(String),
}

impl fmt::Display for RssSummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RssSummaryError::HttpError(e) => write!(f, "HTTP error: {}", e),
            RssSummaryError::EnvVarError(e) => write!(f, "Environment variable error: {}", e),
            RssSummaryError::JsonError(e) => write!(f, "JSON error: {}", e),
            RssSummaryError::Base64Error(e) => write!(f, "Base64 error: {}", e),
            RssSummaryError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
            RssSummaryError::SummaryError(e) => write!(f, "Summary error: {}", e),
        }
    }
}

impl Error for RssSummaryError {}

impl From<env::VarError> for RssSummaryError {
    fn from(err: env::VarError) -> Self {
        RssSummaryError::EnvVarError(err.to_string())
    }
}

impl From<serde_json::Error> for RssSummaryError {
    fn from(err: serde_json::Error) -> Self {
        RssSummaryError::JsonError(err.to_string())
    }
}

impl From<base64::DecodeError> for RssSummaryError {
    fn from(err: base64::DecodeError) -> Self {
        RssSummaryError::Base64Error(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for RssSummaryError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        RssSummaryError::Utf8Error(err.to_string())
    }
}

impl<E: Error + 'static> From<Box<E>> for RssSummaryError {
    fn from(err: Box<E>) -> Self {
        RssSummaryError::SummaryError(err.to_string())
    }
}

/// RSSフィードから要約を取得する
///
/// # Arguments
/// * `rss_data` - RSSチャンネルデータ
pub async fn fetch_rss_summary(rss_data: &Channel) -> Result<ArticlesResponse, RssSummaryError> {
    // RSSデータをモデルに変換
    let rss_data_items: Vec<RssData> = rss_data
        .items
        .iter()
        .map(|item| RssData {
            title: item.title.as_ref(),
            description: item.description.as_ref(),
            link: item.link.as_ref(),
        })
        .collect();

    // プロンプトの取得
    let prompt = match get_decoded_config() {
        Ok(Some(p)) => p,
        Ok(None) => {
            warn!("No summary prompt found, using empty prompt");
            return Err(RssSummaryError::SummaryError("using empty prompt.".to_string()))
        }
        Err(e) => {
            error!("Error decoding config: {}", e);
            return Err(RssSummaryError::SummaryError(e.to_string()));
        }
    };

    // RSSデータをJSON文字列に変換
    let rss_data_str = serde_json::to_string(&rss_data_items)?;

    // Gemini APIリクエストの作成
    let gemini_request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!("{}{}", prompt, rss_data_str),
            }],
        }],
    };

    // Gemini API URLの取得
    let url = env::var("GEMINI_API_URL").unwrap_or_else(|_| {
        error!("GEMINI_API_URL is not set");
        String::new()
    });

    if url.is_empty() {
        return Err(RssSummaryError::EnvVarError(
            "GEMINI_API_URL is empty".to_string(),
        ));
    }

    // HTTPクライアントの初期化
    let http_client = HttpClientImpl::new();

    // Gemini APIへのリクエスト
    let response: GeminiResponse = http_client
        .post_with_response(&url, &gemini_request_body)
        .await
        .map_err(|e| RssSummaryError::HttpError(e.to_string()))?;

    // レスポンスからサマリーを抽出
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

    // サマリーの取得結果を返す
    summary.ok_or_else(|| {
        RssSummaryError::SummaryError("Failed to extract summary from response".to_string())
    })
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
