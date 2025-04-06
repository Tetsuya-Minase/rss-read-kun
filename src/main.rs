use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use log::{error, info};
use std::env;
use std::sync::Arc;

use crate::application::use_case::fetch_and_summarize::FetchAndSummarizeUseCase;
use crate::domain::rss_summary::RssSummaryService;
use crate::infrastructure::discord::notification_service::DiscordNotificationService;
use crate::infrastructure::event::in_memory_event_publisher::{InMemoryEventPublisher, LoggingEventSubscriber};
use crate::infrastructure::gemini::ai_service::GeminiAiService;
use crate::infrastructure::http_client::HttpClientImpl;
use crate::infrastructure::repository::http_rss_repository::HttpRssRepository;
use crate::presentation::http::handlers::handle_get_request;

mod application;
mod domain;
mod infrastructure;
mod presentation;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ロガーの初期化
    env_logger::init();
    info!("Starting RSS Read Kun server...");

    // 環境変数の読み込み
    if let Err(e) = dotenv() {
        error!("Failed to load .env file: {}", e);
    }

    // 依存関係の設定
    let http_client = HttpClientImpl::new();
    
    // RSSリポジトリの初期化
    let rss_repository = HttpRssRepository::new(http_client.clone());
    
    // Gemini APIのURLを取得
    let gemini_url = env::var("GEMINI_API_URL").unwrap_or_else(|_| {
        error!("GEMINI_API_URL is not set");
        String::new()
    });
    
    // AIサービスの初期化
    let _ai_service = GeminiAiService::new(http_client.clone(), gemini_url);
    
    // RSSサマリーサービスの初期化
    let summary_service = crate::application::rss_summary_service::RssSummaryServiceImpl::new(http_client.clone());
    
    // Discord通知サービスの初期化
    let discord_url = env::var("DISCORD_WEBHOOK_URL").unwrap_or_else(|_| {
        error!("DISCORD_WEBHOOK_URL is not set");
        String::new()
    });
    
    let notification_service = DiscordNotificationService::new(http_client.clone(), discord_url);
    
    // イベントパブリッシャーの初期化
    let event_publisher = InMemoryEventPublisher::new();
    event_publisher.add_subscriber(LoggingEventSubscriber);
    
    // ユースケースの初期化
    let use_case = Arc::new(FetchAndSummarizeUseCase::new(
        rss_repository,
        summary_service,
        notification_service,
        event_publisher,
    ));

    // サーバーの起動
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .service(handle_get_request)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
