use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use log::{error, info};
use std::env;

use crate::application::discord_service;
use crate::infrastructure::http_client::{HttpClient, HttpClientImpl};

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
    let post_data = discord_service::to_post_data(&rss_summary);

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
