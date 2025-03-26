use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use std::env;
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
    let _ = http_client::post(&discord_url, &post_data).await;
    HttpResponse::NoContent()
}

/// Format posted data from RSS data list
///
/// # Arguments
/// * `data_list` - RssData list
fn to_post_data(articles_response: &ArticlesResponse) -> model::embed::EmbedData {
    let embed_fields: Vec<model::embed::EmbedField> = articles_response.data.summary.iter().map(|category| {
        let category_name = category.category_map.keys().next().unwrap().clone();
        let value_text = category.category_map.values().map(|category_details| {
            category_details.articles.iter().map(|article| {
                format!("{}\n[{}]({})", article.description, article.title, article.link)
            }).collect::<Vec<String>>().join("\n")
        }).collect::<Vec<String>>().join("\n");
        model::embed::EmbedField{name: category_name, value: value_text}
    }).collect();
    let embeds = [model::embed::Embed {title: String::from("Zenn trend feed"), url: String::from("https://zenn.dev"), fields: embed_fields}].to_vec();
    model::embed::EmbedData{embeds}
}
