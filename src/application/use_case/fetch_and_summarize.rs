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
    /// * `notification_limit` - 通知の制限数
    pub async fn execute(
        &self,
        feed_url: &str,
        notification_limit: usize,
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

        // 通知データの作成と制限
        let notifications = self.create_notifications(&summary, notification_limit);

        // 通知の送信
        self.notification_service
            .send_notifications(notifications.clone())
            .await
            .map_err(|e| {
                error!("Failed to send notifications: {}", e);
                AppError::NotificationError(e.to_string())
            })?;

        // イベント発行: 通知送信
        self.event_publisher.publish(RssEvent::NotificationSent {
            count: notifications.len(),
        });

        info!("Successfully processed RSS feed and sent notifications");
        Ok(())
    }

    /// 通知データを作成し、制限する
    ///
    /// # Arguments
    /// * `summary` - RSSサマリー
    /// * `limit` - 制限数
    fn create_notifications(
        &self,
        summary: &ArticlesResponse,
        limit: usize,
    ) -> Vec<Notification> {
        let notifications = summary
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
                                let notification_fields = category_details
                                    .articles
                                    .iter()
                                    .map(|article| {
                                        let value_string = format!(
                                            "{}\n[この記事を読む]({})",
                                            article.description, article.link
                                        );
                                        NotificationField {
                                            name: article.title.clone(),
                                            value: value_string
                                        }
                                        // Notification {
                                        //     title: category_name.clone(),
                                        //     fields: vec![crate::domain::notification::NotificationField {
                                        //         name: article.title.clone(),
                                        //         value: value_string,
                                        //     }],
                                        // }
                                    })
                                    .collect::<Vec<_>>();
                                
                                Notification {
                                    title: category_name.clone(),
                                    fields: notification_fields
                                }
                            })
                    })
                    .into_iter()
                    .flatten()
            })
            .collect::<Vec<Notification>>();

        // 通知データの制限
        if notifications.len() > limit {
            notifications[..limit].to_vec()
        } else {
            notifications
        }
    }
}
