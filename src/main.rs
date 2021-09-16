use std::error::Error;
use rss::{Channel};
use reqwest::{Client, header};
use serde::{Serialize};
use actix_web::{get, post, put, delete, App, HttpResponse, HttpServer, Responder};

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

#[get("/")]
async fn get() -> impl Responder {
    let rss_data = read_rss().await.unwrap();
    let data_list:Vec<RssData>  = to_rss_data_list(&rss_data);
    let post_data = to_post_data(&data_list);
    send_rss(post_data).await;
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get)
    })
        // port8080で起動
        .bind("0.0.0.0:8080")?
        .run()
        .await
}

/// Returns rss raw data or error
async fn read_rss() -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get("https://zenn.dev/feed")
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

/// Return required Rss data from Channel
/// 
/// # Arguments
/// * `rss_data` - Channel
fn to_rss_data_list(rss_data: &Channel) -> Vec<RssData> {
    // 文字数制限で落ちる可能性があるので10件程度に丸める
    let (split_data, _) = rss_data.items.split_at(10);
    split_data.iter().map(|item| RssData{title: item.title.as_ref(), description: item.description.as_ref(), link: item.link.as_ref()}).collect()
}

/// Format posted data from RSS data list
/// 
/// # Arguments
/// * `data_list` - RssData list
fn to_post_data(data_list: &Vec<RssData>) -> EmbedData {
    let embed_fields: Vec<EmbedField> = data_list.iter().map(|data| {
        let value_text: String = format!("{}\n[この記事を読む]({})", data.description.unwrap(), data.link.unwrap());
        EmbedField{name: data.title.unwrap().clone(), value: value_text}
    }).collect();
    let embeds = [Embed {title: String::from("Zenn trend feed"), url: String::from("https://zenn.dev"), fields: embed_fields}].to_vec();
    EmbedData{embeds}
}

/// post rss data
/// 
/// # Arguments
/// * `data` - posted embed data
async fn send_rss(data: EmbedData) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = env!("DISCORD_WEBHOOK_URL");
    client.post(url).header(header::CONTENT_TYPE, "application/json").json(&data).send().await?;
    Ok(())
}
