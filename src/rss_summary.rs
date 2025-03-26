use crate::http_client;
use crate::model::gemini_request::{Content, GeminiRequest, Part};
use crate::model::gemini_response::GeminiResponse;
use crate::model::rss_data::RssData;
use crate::model::rss_summary::ArticlesResponse;
use base64::{engine::general_purpose, Engine as _};
use rss::Channel;
use std::env;
use std::error::Error;

pub async fn fetch_rss_summary(rss_data: &Channel) -> ArticlesResponse {
    let rss_data_items: Vec<RssData> = rss_data
        .items
        .iter()
        .map(|item| RssData {
            title: item.title.as_ref(),
            description: item.description.as_ref(),
            link: item.link.as_ref(),
        })
        .collect();
    let prompt = get_decoded_config().unwrap_or_else(|e| {
        eprintln!("Error decoding config: {}", e);
        None
    });
    let rss_data_str = serde_json::to_string(&rss_data_items).unwrap();
    let gemini_request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!("{}{}", prompt.unwrap(), rss_data_str),
            }],
        }],
    };
    let url = env::var("GEMINI_API_URL").unwrap_or(String::from(""));
    let response: GeminiResponse = http_client::post_with_response(&url, &gemini_request_body)
        .await
        .unwrap();
    let summary: Option<ArticlesResponse> = response
        .candidates
        .iter()
        .filter_map(|candidate| {
            candidate
                .content
                .parts
                .iter()
                .filter_map(|part| {
                    // code blockを削除
                    let part_text = part.text.replace("```json", "").replace("```", "");
                    match serde_json::from_str(&part_text) {
                        Ok(summary) => Some(summary),
                        Err(_) => None,
                    }
                })
                .next()
        })
        .next();
    match summary {
        Some(articles_response) => {
            articles_response
        }
        None => {
            // 取得できなかった場合のエラーハンドリング
            panic!("Failed to get summary");
        }
    }
}

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
