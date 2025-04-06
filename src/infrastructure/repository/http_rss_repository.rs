use async_trait::async_trait;
use log::error;
use rss::Channel;

use crate::domain::model::rss_data::RssData;
use crate::domain::repository::rss_repository::{RssRepository, RssRepositoryError};
use crate::infrastructure::http_client::{HttpClient, HttpClientError};

/// HTTPを使用したRSSリポジトリの実装
pub struct HttpRssRepository<T: HttpClient> {
    http_client: T,
}

impl<T: HttpClient> HttpRssRepository<T> {
    /// 新しいHTTP RSSリポジトリを作成する
    ///
    /// # Arguments
    /// * `http_client` - HTTPクライアント
    pub fn new(http_client: T) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl<T: HttpClient + Send + Sync + 'static> RssRepository for HttpRssRepository<T> {
    async fn fetch_feed(&self, url: &str) -> Result<Channel, RssRepositoryError> {
        match self.http_client.get(url).await {
            Ok(channel) => Ok(channel),
            Err(e) => {
                error!("Failed to fetch RSS feed: {}", e);
                Err(RssRepositoryError::FetchError(e.to_string()))
            }
        }
    }

    fn convert_to_rss_data(&self, rss_channel: &Channel) -> Vec<RssData> {
        rss_channel
            .items
            .iter()
            .map(|item| RssData {
                title: item.title.as_ref().cloned(),
                description: item.description.as_ref().cloned(),
                link: item.link.as_ref().cloned(),
            })
            .collect()
    }
}

impl From<HttpClientError> for RssRepositoryError {
    fn from(error: HttpClientError) -> Self {
        match error {
            HttpClientError::RequestError(e) => RssRepositoryError::FetchError(e.to_string()),
            HttpClientError::ParseError(e) => RssRepositoryError::ParseError(e),
            HttpClientError::ResponseError(e) => RssRepositoryError::FetchError(e),
        }
    }
}
