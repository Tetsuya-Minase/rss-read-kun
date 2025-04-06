use reqwest::{header, Client};
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// HTTPクライアントのエラー型
#[derive(Debug)]
pub enum HttpClientError {
    RequestError(reqwest::Error),
    ParseError(String),
    ResponseError(String),
}

impl fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpClientError::RequestError(e) => write!(f, "Request error: {}", e),
            HttpClientError::ParseError(e) => write!(f, "Parse error: {}", e),
            HttpClientError::ResponseError(e) => write!(f, "Response error: {}", e),
        }
    }
}

impl Error for HttpClientError {}

impl From<reqwest::Error> for HttpClientError {
    fn from(err: reqwest::Error) -> Self {
        HttpClientError::RequestError(err)
    }
}

impl From<rss::Error> for HttpClientError {
    fn from(err: rss::Error) -> Self {
        HttpClientError::ParseError(err.to_string())
    }
}

impl From<serde_json::Error> for HttpClientError {
    fn from(err: serde_json::Error) -> Self {
        HttpClientError::ParseError(err.to_string())
    }
}

/// HTTPクライアントのトレイト
pub trait HttpClient {
    /// GETリクエストを送信し、RSSチャンネルを取得する
    ///
    /// # Arguments
    /// * `url` - リクエスト先のURL
    async fn get_rss(&self, url: &str) -> Result<Channel, HttpClientError>;

    /// POSTリクエストを送信する
    ///
    /// # Arguments
    /// * `url` - リクエスト先のURL
    /// * `body` - リクエストボディ
    async fn post<T: Serialize + ?Sized>(&self, url: &str, body: &T) -> Result<(), HttpClientError>;

    /// POSTリクエストを送信し、レスポンスを取得する
    ///
    /// # Arguments
    /// * `url` - リクエスト先のURL
    /// * `body` - リクエストボディ
    async fn post_with_response<T: Serialize + ?Sized, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, HttpClientError>;
}

/// HTTPクライアントの実装
#[derive(Clone)]
pub struct HttpClientImpl {
    client: Client,
}

impl HttpClientImpl {
    /// 新しいHTTPクライアントを作成する
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for HttpClientImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for HttpClientImpl {
    async fn get_rss(&self, url: &str) -> Result<Channel, HttpClientError> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        let channel = Channel::read_from(&content[..])?;
        Ok(channel)
    }

    async fn post<T: Serialize + ?Sized>(&self, url: &str, body: &T) -> Result<(), HttpClientError> {
        let response = self
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_message = match response.text().await {
                Ok(text) => format!("Status: {}, Body: {}", status, text),
                Err(_) => format!("Status: {}, Body: <unable to read>", status),
            };
            return Err(HttpClientError::ResponseError(error_message));
        }

        Ok(())
    }

    async fn post_with_response<T: Serialize + ?Sized, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, HttpClientError> {
        let response = self
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_message = match response.text().await {
                Ok(text) => format!("Status: {}, Body: {}", status, text),
                Err(_) => format!("Status: {}, Body: <unable to read>", status),
            };
            return Err(HttpClientError::ResponseError(error_message));
        }

        let response_text = response.text().await?;
        let response_json = serde_json::from_str(&response_text)?;
        Ok(response_json)
    }
}
