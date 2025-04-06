use crate::domain::rss_summary::error::RssSummaryError;
use crate::domain::model::rss_summary::ArticlesResponse;
use rss::Channel;

/// RSSサマリーサービスのトレイト
pub trait RssSummaryService {
    /// RSSフィードから要約を取得する
    ///
    /// # Arguments
    /// * `rss_channel` - RSSチャンネルデータ
    async fn fetch_summary(
        &self,
        rss_channel: &Channel,
    ) -> Result<ArticlesResponse, RssSummaryError>;
}
