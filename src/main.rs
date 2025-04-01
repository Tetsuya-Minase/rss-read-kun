use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use std::env;
use log::{warn};
use crate::model::embed::{Embed, EmbedData, EmbedField};
use crate::model::rss_summary::ArticlesResponse;

pub mod http_client;
pub mod model;
pub mod rss_summary;

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


/// handling get request 
#[get("/")]
async fn get() -> impl Responder {
    dotenv().ok();
    let rss_data = http_client::get("https://zenn.dev/feed").await.unwrap();
    let rss_summary = rss_summary::fetch_rss_summary(&rss_data).await;
    let post_data = to_post_data(&rss_summary);
    let discord_url = env::var("DISCORD_WEBHOOK_URL").unwrap_or(String::from(""));
    let discord_response = http_client::post(&discord_url, &post_data).await;
    match discord_response {
        Ok(_) => println!("Success to post data to discord"),
        Err(e) => println!("Failed to post data to discord: {}", e)
    }
    HttpResponse::NoContent()
}

/// Format posted data from RSS data list
///
/// # Arguments
/// * `data_list` - RssData list
fn to_post_data(articles_response: &ArticlesResponse) -> EmbedData {
    let embed_fields: Vec<Embed> = articles_response.data.summary.iter().flat_map(|category| {
        // カテゴリ名を取得
        let category_name = category.category_map.keys().next().unwrap().clone();
        // カテゴリ内の記事を取得
        category.category_map.values().map(move |category_details| {
            let embed_field: Vec<EmbedField> = category_details.articles.iter().map({
                move |article| {
                    let value_string = format!("{}\n[この記事を読む]({})", article.description, article.link);
                    EmbedField { name: article.title.clone(), value: value_string }
                }
            }).collect();
            Embed { title: category_name.clone(), fields: embed_field }
        })
    }).collect();
    if embed_fields.len() > 10 {
        warn!("Embed fields exceed 10, truncating to 10(count: {})", embed_fields.len());
        let embed_fields = &embed_fields[..10];
        EmbedData { embeds: embed_fields.to_vec() }
    } else {
        EmbedData { embeds: embed_fields }
    }
}
