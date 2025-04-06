use log::info;
use std::sync::{Arc, Mutex};

use crate::domain::event::rss_events::{EventPublisher, EventSubscriber, RssEvent};

/// インメモリイベントパブリッシャーの実装
pub struct InMemoryEventPublisher {
    subscribers: Arc<Mutex<Vec<Box<dyn EventSubscriber + Send + Sync>>>>,
}

impl InMemoryEventPublisher {
    /// 新しいインメモリイベントパブリッシャーを作成する
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// サブスクライバーを追加する
    ///
    /// # Arguments
    /// * `subscriber` - 追加するサブスクライバー
    pub fn add_subscriber<S>(&self, subscriber: S)
    where
        S: EventSubscriber + Send + Sync + 'static,
    {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(Box::new(subscriber));
    }
}

impl Default for InMemoryEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisher for InMemoryEventPublisher {
    fn publish(&self, event: RssEvent) {
        // イベントの種類をログに出力
        match &event {
            RssEvent::FeedFetched { url, .. } => {
                info!("Event: FeedFetched from {}", url);
            }
            RssEvent::DataConverted { items } => {
                info!("Event: DataConverted with {} items", items.len());
            }
            RssEvent::SummaryGenerated { summary } => {
                info!(
                    "Event: SummaryGenerated with {} articles",
                    summary.data.total
                );
            }
            RssEvent::NotificationSent { count } => {
                info!("Event: NotificationSent with {} notifications", count);
            }
        }

        // 全てのサブスクライバーにイベントを通知
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            subscriber.handle(&event);
        }
    }
}

/// ロギングイベントサブスクライバーの実装
pub struct LoggingEventSubscriber;

impl EventSubscriber for LoggingEventSubscriber {
    fn handle(&self, event: &RssEvent) {
        match event {
            RssEvent::FeedFetched { url, channel } => {
                info!(
                    "LoggingEventSubscriber: RSS feed fetched from {} with {} items",
                    url,
                    channel.items.len()
                );
            }
            RssEvent::DataConverted { items } => {
                info!(
                    "LoggingEventSubscriber: RSS data converted with {} items",
                    items.len()
                );
            }
            RssEvent::SummaryGenerated { summary } => {
                info!(
                    "LoggingEventSubscriber: RSS summary generated with {} articles in {} categories",
                    summary.data.total,
                    summary.data.category_count()
                );
            }
            RssEvent::NotificationSent { count } => {
                info!(
                    "LoggingEventSubscriber: {} notifications sent",
                    count
                );
            }
        }
    }
}
