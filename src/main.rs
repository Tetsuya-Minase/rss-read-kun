use rss::{Channel};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};

pub mod http_client;
pub mod model;

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
    let rss_data = http_client::get("https://zenn.dev/feed").await.unwrap();
    let data_list:Vec<model::rss_data::RssData>  = to_rss_data_list(&rss_data);
    let post_data = to_post_data(&data_list);
    let discord_url = env!("DISCORD_WEBHOOK_URL");
    let _ = http_client::post(discord_url, &post_data).await;
    HttpResponse::NoContent()
}

/// Return required Rss data from Channel
/// 
/// # Arguments
/// * `rss_data` - Channel
fn to_rss_data_list(rss_data: &Channel) -> Vec<model::rss_data::RssData> {
    // 文字数制限で落ちる可能性があるので10件程度に丸める
    let (split_data, _) = rss_data.items.split_at(10);
    split_data.iter().map(|item| model::rss_data::RssData{title: item.title.as_ref(), description: item.description.as_ref(), link: item.link.as_ref()}).collect()
}

/// Format posted data from RSS data list
/// 
/// # Arguments
/// * `data_list` - RssData list
fn to_post_data(data_list: &Vec<model::rss_data::RssData>) -> model::embed::EmbedData {
    let embed_fields: Vec<model::embed::EmbedField> = data_list.iter().map(|data| {
        let value_text: String = format!("{}\n[この記事を読む]({})", data.description.unwrap(), data.link.unwrap());
        model::embed::EmbedField{name: data.title.unwrap().clone(), value: value_text}
    }).collect();
    let embeds = [model::embed::Embed {title: String::from("Zenn trend feed"), url: String::from("https://zenn.dev"), fields: embed_fields}].to_vec();
    model::embed::EmbedData{embeds}
}
