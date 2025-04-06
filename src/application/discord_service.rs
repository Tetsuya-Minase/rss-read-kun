use log::warn;
use crate::model::embed::{Embed, EmbedData, EmbedField};
use crate::model::rss_summary::ArticlesResponse;

/// RSSの要約データからDiscord用のEmbedデータを作成する
///
/// # Arguments
/// * `articles_response` - RSSの要約データ
pub fn to_post_data(articles_response: &ArticlesResponse) -> EmbedData {
    let embed_fields: Vec<Embed> = articles_response
        .data
        .summary
        .iter()
        .flat_map(|category| {
            // カテゴリ名を取得
            category
                .category_map
                .keys()
                .next()
                .map(|category_name| {
                    // カテゴリ内の記事を取得
                    category
                        .category_map
                        .values()
                        .map(move |category_details| {
                            let embed_field: Vec<EmbedField> = category_details
                                .articles
                                .iter()
                                .map(|article| {
                                    let value_string = format!(
                                        "{}\n[この記事を読む]({})",
                                        article.description, article.link
                                    );
                                    EmbedField {
                                        name: article.title.clone(),
                                        value: value_string,
                                    }
                                })
                                .collect();
                            Embed {
                                title: category_name.clone(),
                                fields: embed_field,
                            }
                        })
                })
                .into_iter()
                .flatten()
        })
        .collect();

    // Discordの制限に合わせて10個までに制限
    if embed_fields.len() > 10 {
        warn!(
            "Embed fields exceed 10, truncating to 10 (count: {})",
            embed_fields.len()
        );
        let truncated_fields = &embed_fields[..10];
        EmbedData {
            embeds: truncated_fields.to_vec(),
        }
    } else {
        EmbedData {
            embeds: embed_fields,
        }
    }
}
