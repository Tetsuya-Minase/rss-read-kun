use log::{error, info};

use crate::domain::event::rss_events::{EventPublisher, RssEvent};
use crate::domain::model::rss_summary::ArticlesResponse;
use crate::domain::notification::{Notification, NotificationField, NotificationService};
use crate::domain::repository::rss_repository::RssRepository;
use crate::domain::rss_summary::{RssSummaryError, RssSummaryService};

use std::fmt;
use std::error::Error;

/// アプリケーションエラー型
#[derive(Debug)]
pub enum AppError {
    RssError(String),
    SummaryError(String),
    NotificationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::RssError(e) => write!(f, "RSS error: {}", e),
            AppError::SummaryError(e) => write!(f, "Summary error: {}", e),
            AppError::NotificationError(e) => write!(f, "Notification error: {}", e),
        }
    }
}

impl Error for AppError {}

impl From<RssSummaryError> for AppError {
    fn from(error: RssSummaryError) -> Self {
        AppError::SummaryError(error.to_string())
    }
}

/// RSSフィードを取得し、要約して通知するユースケース
pub struct FetchAndSummarizeUseCase<R, S, N, E>
where
    R: RssRepository,
    S: RssSummaryService,
    N: NotificationService,
    E: EventPublisher,
{
    rss_repository: R,
    summary_service: S,
    notification_service: N,
    event_publisher: E,
}

impl<R, S, N, E> FetchAndSummarizeUseCase<R, S, N, E>
where
    R: RssRepository,
    S: RssSummaryService,
    N: NotificationService,
    E: EventPublisher,
{
    /// 新しいユースケースを作成する
    ///
    /// # Arguments
    /// * `rss_repository` - RSSリポジトリ
    /// * `summary_service` - RSSサマリーサービス
    /// * `notification_service` - 通知サービス
    /// * `event_publisher` - イベントパブリッシャー
    pub fn new(
        rss_repository: R,
        summary_service: S,
        notification_service: N,
        event_publisher: E,
    ) -> Self {
        Self {
            rss_repository,
            summary_service,
            notification_service,
            event_publisher,
        }
    }

    /// RSSフィードを取得し、要約して通知する
    ///
    /// # Arguments
    /// * `feed_url` - RSSフィードのURL
    pub async fn execute(
        &self,
        feed_url: &str,
    ) -> Result<(), AppError> {
        let result = self.execute_inner(feed_url).await;
        if let Err(ref err) = result {
            let error_notification =
                crate::application::error_notification::create_error_notification(err);
            if let Err(send_err) = self
                .notification_service
                .send_notifications(crate::application::notification::discord_limits::ValidatedNotifications(vec![error_notification]))
                .await
            {
                error!("Failed to send error notification: {}", send_err);
            }
        }
        result
    }

    /// RSS処理の内部実行本体
    async fn execute_inner(
        &self,
        feed_url: &str,
    ) -> Result<(), AppError> {
        // RSSフィードの取得
        let rss_channel = self
            .rss_repository
            .fetch_feed(feed_url)
            .await
            .map_err(|e| {
                error!("Failed to fetch RSS feed: {}", e);
                AppError::RssError(e.to_string())
            })?;

        // イベント発行: フィード取得
        self.event_publisher.publish(RssEvent::FeedFetched {
            url: feed_url.to_string(),
            channel: rss_channel.clone(),
        });

        // RSSデータをモデルに変換
        let rss_data_items = self.rss_repository.convert_to_rss_data(&rss_channel);

        // イベント発行: データ変換
        self.event_publisher.publish(RssEvent::DataConverted {
            items: rss_data_items.clone(),
        });

        // RSSサマリーの取得
        let summary = self.summary_service.fetch_summary(&rss_channel).await?;

        // イベント発行: サマリー生成
        self.event_publisher.publish(RssEvent::SummaryGenerated {
            summary: summary.clone(),
        });

        // 通知データの作成と制限 (plan_digest に委譲)
        let digest_plan = crate::application::notification::discord_limits::plan_digest(&summary);
        
        let notif_count = digest_plan.notifications.0.len();

        // 通知の送信
        self.notification_service
            .send_notifications(digest_plan.notifications)
            .await
            .map_err(|e| {
                error!("Failed to send notifications: {}", e);
                AppError::NotificationError(e.to_string())
            })?;

        // イベント発行: 通知送信
        self.event_publisher.publish(RssEvent::NotificationSent {
            count: notif_count,
        });

        info!("Successfully processed RSS feed and sent notifications");
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::domain::model::rss_data::RssData;
    use crate::domain::model::rss_summary::{Article, ArticlesData, Category, Highlight};
    use crate::domain::notification::NotificationError;
    use crate::domain::repository::rss_repository::RssRepositoryError;
    use rss::Channel;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Mutex;

    struct MockRssRepository {
        feed_result: Mutex<Option<Result<Channel, RssRepositoryError>>>,
    }

    impl MockRssRepository {
        fn new(result: Result<Channel, RssRepositoryError>) -> Self {
            Self {
                feed_result: Mutex::new(Some(result)),
            }
        }
    }

    #[async_trait]
    impl RssRepository for MockRssRepository {
        async fn fetch_feed(&self, _url: &str) -> Result<Channel, RssRepositoryError> {
            self.feed_result
                .lock()
                .unwrap()
                .take()
                .expect("fetch_feed called more than expected")
        }

        fn convert_to_rss_data(&self, _rss_channel: &Channel) -> Vec<RssData> {
            vec![]
        }
    }

    struct MockRssSummaryService {
        summary_result: Mutex<Option<Result<ArticlesResponse, RssSummaryError>>>,
    }

    impl MockRssSummaryService {
        fn new(result: Result<ArticlesResponse, RssSummaryError>) -> Self {
            Self {
                summary_result: Mutex::new(Some(result)),
            }
        }
    }

    impl RssSummaryService for MockRssSummaryService {
        async fn fetch_summary(
            &self,
            _rss_channel: &Channel,
        ) -> Result<ArticlesResponse, RssSummaryError> {
            self.summary_result
                .lock()
                .unwrap()
                .take()
                .expect("fetch_summary called more than expected")
        }
    }

    struct MockNotificationService {
        calls: Mutex<Vec<Vec<Notification>>>,
        responses: Mutex<VecDeque<Result<(), NotificationError>>>,
    }

    impl MockNotificationService {
        fn new(responses: Vec<Result<(), NotificationError>>) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                responses: Mutex::new(VecDeque::from(responses)),
            }
        }

        fn get_calls(&self) -> Vec<Vec<Notification>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl NotificationService for MockNotificationService {
        async fn send_notifications(
            &self,
            notifications: crate::application::notification::discord_limits::ValidatedNotifications,
        ) -> Result<(), NotificationError> {
            self.calls.lock().unwrap().push(notifications.0);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or(Ok(()))
        }
    }

    struct MockEventPublisher {
        events: Mutex<Vec<RssEvent>>,
    }

    impl MockEventPublisher {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
    }

    impl EventPublisher for MockEventPublisher {
        fn publish(&self, event: RssEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    fn create_dummy_articles_response() -> ArticlesResponse {
        ArticlesResponse {
            message: "Success".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![Category {
                    name: "Tech".to_string(),
                    articles: vec![Article {
                        title: "Test Article".to_string(),
                        summary: "Description".to_string(),
                        link: "https://example.com/article".to_string(),
                    }],
                }],
            },
        }
    }

    #[actix_web::test]
    async fn test_t7_success_does_not_send_error_notification() {
        let repo = MockRssRepository::new(Ok(Channel::default()));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        assert!(result.is_ok());
        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 1, "send_notifications should be called exactly once for articles");
        assert!(calls[0][0].title.contains("Zennトレンド"));
    }

    #[actix_web::test]
    async fn test_t8_rss_error_sends_notification() {
        let repo = MockRssRepository::new(Err(RssRepositoryError::FetchError("404 Not Found".to_string())));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        assert!(matches!(result, Err(AppError::RssError(_))));
        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 1, "send_notifications should be called once for error notification");
        assert_eq!(calls[0][0].title, "⚠️ RSS処理でエラーが発生しました");
        assert_eq!(calls[0][0].fields[0].name, "RSSフィード取得エラー");
        assert!(calls[0][0].fields[0].value.contains("404 Not Found"));
    }

    #[actix_web::test]
    async fn test_t9_summary_error_sends_notification() {
        let repo = MockRssRepository::new(Ok(Channel::default()));
        let summary = MockRssSummaryService::new(Err(RssSummaryError::SummaryError("using empty prompt.".to_string())));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        assert!(matches!(result, Err(AppError::SummaryError(_))));
        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 1, "send_notifications should be called once for error notification");
        assert_eq!(calls[0][0].title, "⚠️ RSS処理でエラーが発生しました");
        assert_eq!(calls[0][0].fields[0].name, "要約生成エラー");
        assert!(calls[0][0].fields[0].value.contains("using empty prompt."));
    }

    #[actix_web::test]
    async fn test_t10_article_notification_error_tries_error_notification() {
        let repo = MockRssRepository::new(Ok(Channel::default()));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        let notif = MockNotificationService::new(vec![
            Err(NotificationError::SendError("Status: 429".to_string())),
            Ok(()),
        ]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        assert!(matches!(result, Err(AppError::NotificationError(_))));
        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 2, "send_notifications should be called twice (article + error)");
        assert!(calls[0][0].title.contains("Zennトレンド"));
        assert_eq!(calls[1][0].title, "⚠️ RSS処理でエラーが発生しました");
        assert_eq!(calls[1][0].fields[0].name, "通知送信エラー");
    }

    #[actix_web::test]
    async fn test_t11_error_notification_failure_returns_original_error() {
        let repo = MockRssRepository::new(Err(RssRepositoryError::FetchError("connection failed".to_string())));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        // エラー通知の送信も失敗する設定
        let notif = MockNotificationService::new(vec![
            Err(NotificationError::SendError("Webhook down".to_string())),
        ]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        // エラー通知失敗でも panic せず、元の RssError が返ること（P2不変条件）
        match result {
            Err(AppError::RssError(msg)) => assert!(msg.contains("connection failed")),
            _ => panic!("Expected RssError, got {:?}", result),
        }
    }

    #[actix_web::test]
    async fn test_t12_long_error_message_is_truncated_within_1024() {
        let long_msg = "x".repeat(1025);
        let repo = MockRssRepository::new(Err(RssRepositoryError::FetchError(long_msg)));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let _ = use_case.execute("https://example.com/feed").await;

        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0][0].fields[0].value.chars().count(), 1024);
    }

    #[actix_web::test]
    async fn test_p2_p3_invariants() {
        // P3: send_notifications は最大2回
        // 成功時: 1回
        let repo = MockRssRepository::new(Ok(Channel::default()));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();
        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let _ = use_case.execute("https://example.com/feed").await;
        assert!(use_case.notification_service.get_calls().len() <= 2);

        // 記事通知エラー時: 2回（記事1回 + エラー1回）
        let repo = MockRssRepository::new(Ok(Channel::default()));
        let summary = MockRssSummaryService::new(Ok(create_dummy_articles_response()));
        let notif = MockNotificationService::new(vec![
            Err(NotificationError::SendError("Fail".to_string())),
            Ok(()),
        ]);
        let event = MockEventPublisher::new();
        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let _ = use_case.execute("https://example.com/feed").await;
        assert_eq!(use_case.notification_service.get_calls().len(), 2);
    }

    #[actix_web::test]
    async fn test_t36_empty_notifications_still_calls_service() {
        let repo = MockRssRepository::new(Ok(Channel::default()));
        let mut empty_response = create_dummy_articles_response();
        empty_response.data.categories.clear();
        empty_response.data.total = 0;
        
        let summary = MockRssSummaryService::new(Ok(empty_response));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        assert!(result.is_ok());
        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].len(), 1, "Only header");
        assert_eq!(calls[0][0].title, "📰 Zennトレンド 0件");
    }

    #[actix_web::test]
    async fn test_t38_execute_does_not_truncate_but_relies_on_plan_digest() {
        let repo = MockRssRepository::new(Ok(Channel::default()));
        
        let mut categories = Vec::new();
        for i in 0..12 {
            categories.push(crate::domain::model::rss_summary::Category {
                name: format!("C{}", i),
                articles: vec![crate::domain::model::rss_summary::Article {
                    title: "A".to_string(),
                    summary: "S".to_string(),
                    link: "L".to_string(),
                }],
            });
        }
        
        let mut response = create_dummy_articles_response();
        response.data.categories = categories;
        response.data.total = 12;
        
        let summary = MockRssSummaryService::new(Ok(response));
        let notif = MockNotificationService::new(vec![Ok(())]);
        let event = MockEventPublisher::new();

        let use_case = FetchAndSummarizeUseCase::new(repo, summary, notif, event);
        let result = use_case.execute("https://example.com/feed").await;

        assert!(result.is_ok());
        let calls = use_case.notification_service.get_calls();
        assert_eq!(calls.len(), 1);
        
        // plan_digest should have capped it at 10 embeds, skipped 2
        let sent_notifs = &calls[0];
        assert_eq!(sent_notifs.len(), 10);
        
        // Header should contain skipped warning
        assert!(sent_notifs[0].description.as_ref().unwrap().contains("⚠️ Discordの制限により3件を除外しました"));
    }
}
