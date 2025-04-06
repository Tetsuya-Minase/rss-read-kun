use crate::domain::model::rss_data::RssData;
use crate::domain::model::rss_summary::ArticlesResponse;
use rss::Channel;

/// RSSイベントを表す列挙型
#[derive(Debug, Clone)]
pub enum RssEvent {
    /// RSSフィードが取得されたイベント
    FeedFetched {
        url: String,
        channel: Channel,
    },
    /// RSSデータが変換されたイベント
    DataConverted {
        items: Vec<RssData>,
    },
    /// RSSサマリーが生成されたイベント
    SummaryGenerated {
        summary: ArticlesResponse,
    },
    /// 通知が送信されたイベント
    NotificationSent {
        count: usize,
    },
}

/// イベントパブリッシャーのトレイト
pub trait EventPublisher {
    /// イベントを発行する
    ///
    /// # Arguments
    /// * `event` - 発行するイベント
    fn publish(&self, event: RssEvent);
}

/// イベントサブスクライバーのトレイト
pub trait EventSubscriber {
    /// イベントを処理する
    ///
    /// # Arguments
    /// * `event` - 処理するイベント
    fn handle(&self, event: &RssEvent);
}
