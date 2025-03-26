use crate::http_client;
use crate::model::gemini_request::{Content, GeminiRequest, Part};
use crate::model::gemini_response::GeminiResponse;
use crate::model::rss_data::RssData;
use crate::model::rss_summary::ArticlesResponse;
use rss::Channel;
use std::env;

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
    let prompt = env::var("SUMMARY_PROMPT").unwrap();
    let rss_data_str = serde_json::to_string(&rss_data_items).unwrap();
    let gemini_request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: format!("{}{}", prompt, rss_data_str),
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
            println!("summary: {:?}", articles_response);
            articles_response
        }
        None => {
            // 取得できなかった場合のエラーハンドリング
            panic!("Failed to get summary");
        }
    }
}
