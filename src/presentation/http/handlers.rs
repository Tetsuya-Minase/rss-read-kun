use actix_web::{get, web, HttpResponse, Responder};
use dotenvy::dotenv;
use log::{error, info};
use std::sync::Arc;

use crate::application::use_case::fetch_and_summarize::{AppError, FetchAndSummarizeUseCase};
use crate::domain::event::rss_events::EventPublisher;
use crate::domain::notification::NotificationService;
use crate::domain::repository::rss_repository::RssRepository;
use crate::domain::rss_summary::RssSummaryService;

type UseCase = FetchAndSummarizeUseCase<
    crate::infrastructure::repository::http_rss_repository::HttpRssRepository<crate::infrastructure::http_client::HttpClientImpl>,
    crate::application::rss_summary_service::RssSummaryServiceImpl<crate::infrastructure::http_client::HttpClientImpl>,
    crate::infrastructure::discord::notification_service::DiscordNotificationService<crate::infrastructure::http_client::HttpClientImpl>,
    crate::infrastructure::event::in_memory_event_publisher::InMemoryEventPublisher
>;

/// RSSフィードを取得し、要約してDiscordに送信するエンドポイント
#[get("/")]
pub async fn handle_get_request(
    use_case: web::Data<Arc<UseCase>>,
) -> impl Responder {
    // 環境変数の読み込み
    if let Err(e) = dotenv() {
        error!("Failed to load .env file: {}", e);
    }

    // RSSフィードのURL
    let rss_feed_url = "https://zenn.dev/feed";

    // ユースケースの実行
    match use_case.execute(rss_feed_url, 10).await {
        Ok(_) => {
            info!("Successfully processed RSS feed");
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            error!("Failed to process RSS feed: {}", e);
            match e {
                AppError::RssError(_) => HttpResponse::InternalServerError().body("Failed to fetch RSS feed"),
                AppError::SummaryError(_) => HttpResponse::InternalServerError().body("Failed to generate summary"),
                AppError::NotificationError(_) => HttpResponse::InternalServerError().body("Failed to send notification"),
            }
        }
    }
}
