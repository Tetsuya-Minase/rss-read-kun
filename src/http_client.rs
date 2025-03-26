use reqwest::{header, Client};
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// send get requests
///
/// # Arguments
/// * `url` - request url
pub async fn get(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

/// send post requests
///
/// # Arguments
/// * `url` - request url
/// * `body` - request body
pub async fn post<T: Serialize>(url: &str, body: &T) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .json(body)
        .send()
        .await?;
    Ok(())
}

/// send post requests and return response
///
/// # Arguments
/// * `url` - request url
/// * `body` - request body
pub async fn post_with_response<T: Serialize, R: for<'de> Deserialize<'de>>(
    url: &str,
    body: &T,
) -> Result<R, Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .json(body)
        .send()
        .await?;
    let response_text = response.text().await?;
    let response_json: R = serde_json::from_str(&response_text)?;
    Ok(response_json)
}
