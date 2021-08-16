use std::error::Error;
use rss::{Channel};
use reqwest::{Client, Response, header};
use serde::{Serialize};

/// RSSのデータを格納する
struct RssData<'a> {
    title:Option<&'a String>,
    description: Option<&'a String>,
    link: Option<&'a String>,
}

#[derive(Serialize, Debug, Clone)]
struct EmbedField {
    name: String,
    value: String
}
#[derive(Serialize, Debug, Clone)]
struct Embed {
    title: String,
    url: String,
    fields: Vec<EmbedField>
}
#[derive(Serialize, Debug)]
struct EmbedData {
    embeds: Vec<Embed>
}

#[tokio::main]
async fn main() {
    let rss_data = read_rss().await.unwrap();
    let data_list:Vec<RssData>  = to_rss_data_list(&rss_data);
    let post_data = to_post_data(&data_list);
    send_rss(post_data).await;
}

/// rss読む
async fn read_rss() -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get("https://zenn.dev/feed")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

/// rssのデータから必要なdataを取得する
fn to_rss_data_list(rss_data: &Channel) -> Vec<RssData> {
    // 文字数制限で落ちる可能性があるので10件程度に丸める
    let (split_data, _) = rss_data.items.split_at(10);
    split_data.iter().map(|item| RssData{title: item.title.as_ref(), description: item.description.as_ref(), link: item.link.as_ref()}).collect()
}

/// Discordにpostする形式に変換する
fn to_post_data(data_list: &Vec<RssData>) -> EmbedData {
    let embed_fields: Vec<EmbedField> = data_list.iter().map(|data| {
        let value_text: String = format!("{}\n[この記事を読む]({})", data.description.unwrap(), data.link.unwrap());
        EmbedField{name: data.title.unwrap().clone(), value: value_text}
    }).collect();
    let embeds = [Embed {title: String::from("Zenn trend feed"), url: String::from("https://zenn.dev"), fields: embed_fields}].to_vec();
    EmbedData{embeds}
}

/// Discordにpostする
async fn send_rss(data: EmbedData) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = env!("DISCORD_WEBHOOK_URL");
    client.post(url).header(header::CONTENT_TYPE, "application/json").json(&data).send().await?;
    Ok(())
}
