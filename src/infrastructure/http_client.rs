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
    fn get(&self, url: &str) -> impl std::future::Future<Output = Result<Channel, HttpClientError>> + Send;

    /// POSTリクエストを送信する
    ///
    /// # Arguments
    /// * `url` - リクエスト先のURL
    /// * `body` - リクエストボディ
    fn post<T: Serialize + ?Sized + Send + Sync>(&self, url: &str, body: &T) -> impl std::future::Future<Output = Result<(), HttpClientError>> + Send;

    /// POSTリクエストを送信し、レスポンスを取得する
    ///
    /// # Arguments
    /// * `url` - リクエスト先のURL
    /// * `body` - リクエストボディ
    fn post_with_response<T: Serialize + ?Sized + Send + Sync, R: for<'de> Deserialize<'de> + Send>(
        &self,
        url: &str,
        body: &T,
    ) -> impl std::future::Future<Output = Result<R, HttpClientError>> + Send;
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
    fn get(&self, url: &str) -> impl std::future::Future<Output = Result<Channel, HttpClientError>> + Send {
        async move {
            let response = self.client.get(url).send().await?;
            let content = response.bytes().await?;
            let channel = Channel::read_from(&content[..])?;
            Ok(channel)
        }
    }

    fn post<T: Serialize + ?Sized + Send + Sync>(&self, url: &str, body: &T) -> impl std::future::Future<Output = Result<(), HttpClientError>> + Send {
        async move {
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
    }

    fn post_with_response<T: Serialize + ?Sized + Send + Sync, R: for<'de> Deserialize<'de> + Send>(
        &self,
        url: &str,
        body: &T,
    ) -> impl std::future::Future<Output = Result<R, HttpClientError>> + Send {
        async move {
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
}
