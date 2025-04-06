use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use log::{error, info, warn};
use std::env;

use crate::infrastructure::http_client::{HttpClient, HttpClientImpl};
use crate::model::embed::{Embed, EmbedData, EmbedField};
use crate::model::rss_summary::ArticlesResponse;

mod application;
mod domain;
mod infrastructure;
mod model;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting RSS Read Kun server...");

    HttpServer::new(|| App::new().service(handle_get_request))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

/// RSSフィードを取得し、要約してDiscordに送信するエンドポイント
#[get("/")]
async fn handle_get_request() -> impl Responder {
    // 環境変数の読み込み
    if let Err(e) = dotenv() {
        error!("Failed to load .env file: {}", e);
    }

    // HTTPクライアントの初期化
    let http_client = HttpClientImpl::new();

    // RSSフィードの取得
    let rss_feed_url = "https://zenn.dev/feed";
    let rss_data = match http_client.get(rss_feed_url).await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to fetch RSS feed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // RSSの要約取得
    let rss_summary = match application::rss_summary::fetch_rss_summary(&rss_data).await {
        Ok(summary) => summary,
        Err(e) => {
            error!("Failed to fetch RSS summary: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Discordに送信するデータの作成
    let post_data = to_post_data(&rss_summary);

    // Discordへの送信
    let discord_url = env::var("DISCORD_WEBHOOK_URL").unwrap_or_else(|_| {
        error!("DISCORD_WEBHOOK_URL is not set");
        String::new()
    });

    if discord_url.is_empty() {
        error!("Discord webhook URL is empty");
        return HttpResponse::InternalServerError().finish();
    }

    match http_client.post(&discord_url, &post_data).await {
        Ok(_) => info!("Successfully posted data to Discord"),
        Err(e) => error!("Failed to post data to Discord: {}", e),
    }

    HttpResponse::NoContent().finish()
}

/// RSSの要約データからDiscord用のEmbedデータを作成する
///
/// # Arguments
/// * `articles_response` - RSSの要約データ
fn to_post_data(articles_response: &ArticlesResponse) -> EmbedData {
    let embed_fields: Vec<Embed> = articles_response
        .data
        .summary
        .iter()
        .flat_map(|category| {
            // カテゴリ名を取得
            category
                .category_map
                .keys()
                .next()
                .map(|category_name| {
                    // カテゴリ内の記事を取得
                    category
                        .category_map
                        .values()
                        .map(move |category_details| {
                            let embed_field: Vec<EmbedField> = category_details
                                .articles
                                .iter()
                                .map(|article| {
                                    let value_string = format!(
                                        "{}\n[この記事を読む]({})",
                                        article.description, article.link
                                    );
                                    EmbedField {
                                        name: article.title.clone(),
                                        value: value_string,
                                    }
                                })
                                .collect();
                            Embed {
                                title: category_name.clone(),
                                fields: embed_field,
                            }
                        })
                })
                .into_iter()
                .flatten()
        })
        .collect();

    // Discordの制限に合わせて10個までに制限
    if embed_fields.len() > 10 {
        warn!(
            "Embed fields exceed 10, truncating to 10 (count: {})",
            embed_fields.len()
        );
        let truncated_fields = &embed_fields[..10];
        EmbedData {
            embeds: truncated_fields.to_vec(),
        }
    } else {
        EmbedData {
            embeds: embed_fields,
        }
    }
}
