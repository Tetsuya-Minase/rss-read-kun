use std::error::Error;
use rss::{Channel};
use reqwest::{Client, header};
use serde::Serialize;

/// send get requests
/// 
/// # Arguments
/// * `url` - request url 
pub async fn get (url: &str) -> Result<Channel, Box<dyn Error>> {
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
    client.post(url).header(header::CONTENT_TYPE, "application/json").json(body).send().await?;
    Ok(())
}

