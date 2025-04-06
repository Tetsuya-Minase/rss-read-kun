use log::warn;
use crate::domain::notification::{Notification, NotificationField};
use crate::domain::rss_summary::model::ArticlesResponse;

/// RSSの要約データから通知データを作成する
///
/// # Arguments
/// * `articles_response` - RSSの要約データ
pub fn create_notifications(articles_response: &ArticlesResponse) -> Vec<Notification> {
    articles_response
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
                            let notification_fields: Vec<NotificationField> = category_details
                                .articles
                                .iter()
                                .map(|article| {
                                    let value_string = format!(
                                        "{}\n[この記事を読む]({})",
                                        article.description, article.link
                                    );
                                    NotificationField {
                                        name: article.title.clone(),
                                        value: value_string,
                                    }
                                })
                                .collect();

                            Notification {
                                title: category_name.clone(),
                                fields: notification_fields,
                            }
                        })
                })
                .into_iter()
                .flatten()
        })
        .collect::<Vec<Notification>>()
}

/// 通知データを制限する
///
/// # Arguments
/// * `notifications` - 通知データのリスト
/// * `limit` - 制限数
pub fn limit_notifications(notifications: Vec<Notification>, limit: usize) -> Vec<Notification> {
    if notifications.len() > limit {
        warn!(
            "Notifications exceed {}, truncating to {} (count: {})",
            limit, limit, notifications.len()
        );
        notifications[..limit].to_vec()
    } else {
        notifications
    }
}
