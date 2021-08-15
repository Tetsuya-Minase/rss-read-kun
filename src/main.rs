use std::error::Error;
use rss::{Channel, Item};

/// RSSのデータを格納する
struct RssData<'a> {
    title:Option<&'a String>,
    description: Option<&'a String>,
    link: Option<&'a String>,
}

#[tokio::main]
async fn main() {
    let rss_data = read_rss().await.unwrap();
    let data_list:Vec<RssData>  = rss_data.items.iter().map(|item| RssData{title: item.title.as_ref(), description: item.description.as_ref(), link: item.link.as_ref()}).collect();
    for item in data_list.iter() {
        println!("title: {:?}", item.title.unwrap());
        println!("description: {:?}", item.description.unwrap());
        println!("link: {:?}", item.link.unwrap());
    }

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
