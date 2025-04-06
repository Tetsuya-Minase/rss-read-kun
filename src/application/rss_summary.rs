use crate::application::rss_summary_service::RssSummaryServiceImpl;
use crate::domain::rss_summary::{RssSummaryError, RssSummaryService};
use crate::domain::rss_summary::model::ArticlesResponse;
use crate::infrastructure::http_client::HttpClientImpl;
use rss::Channel;

/// RSSフィードから要約を取得する
///
/// # Arguments
/// * `rss_data` - RSSチャンネルデータ
pub async fn fetch_rss_summary(rss_data: &Channel) -> Result<ArticlesResponse, RssSummaryError> {
    // HTTPクライアントの初期化
    let http_client = HttpClientImpl::new();
    
    // RSSサマリーサービスの初期化
    let rss_summary_service = RssSummaryServiceImpl::new(http_client);
    
    // RSSサマリーの取得
    rss_summary_service.fetch_summary(rss_data).await
}
