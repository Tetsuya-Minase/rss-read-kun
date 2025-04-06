use rss::Channel;
use async_trait::async_trait;
use std::fmt;
use crate::domain::model::rss_data::RssData;

/// RSSリポジトリのエラー型
#[derive(Debug)]
pub enum RssRepositoryError {
    FetchError(String),
    ParseError(String),
}

impl fmt::Display for RssRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RssRepositoryError::FetchError(e) => write!(f, "Failed to fetch RSS feed: {}", e),
            RssRepositoryError::ParseError(e) => write!(f, "Failed to parse RSS feed: {}", e),
        }
    }
}

impl std::error::Error for RssRepositoryError {}

/// RSSリポジトリのトレイト
#[async_trait]
pub trait RssRepository {
    /// RSSフィードを取得する
    ///
    /// # Arguments
    /// * `url` - RSSフィードのURL
    async fn fetch_feed(&self, url: &str) -> Result<Channel, RssRepositoryError>;

    /// RSSデータをモデルに変換する
    fn convert_to_rss_data(&self, rss_channel: &Channel) -> Vec<RssData>;
}
