use serde::Serialize;
use crate::domain::notification::{Notification, NotificationError, NotificationService};
use crate::infrastructure::http_client::HttpClient;

/// Discord用の通知フィールドを表す構造体
#[derive(Serialize, Debug, Clone)]
struct DiscordEmbedField {
    name: String,
    value: String,
}

/// Discord用の通知を表す構造体
#[derive(Serialize, Debug, Clone)]
struct DiscordEmbed {
    title: String,
    fields: Vec<DiscordEmbedField>,
}

/// Discord用の通知データを表す構造体
#[derive(Serialize, Debug)]
struct DiscordEmbedData {
    embeds: Vec<DiscordEmbed>,
}

/// Discord通知サービスの実装
pub struct DiscordNotificationService<T: HttpClient> {
    http_client: T,
    webhook_url: String,
}

impl<T: HttpClient> DiscordNotificationService<T> {
    /// 新しいDiscord通知サービスを作成する
    ///
    /// # Arguments
    /// * `http_client` - HTTPクライアント
    /// * `webhook_url` - DiscordのWebhook URL
    pub fn new(http_client: T, webhook_url: String) -> Self {
        Self {
            http_client,
            webhook_url,
        }
    }

    /// 通知をDiscord用のデータに変換する
    ///
    /// # Arguments
    /// * `notifications` - 送信する通知のリスト
    fn to_discord_data(&self, notifications: Vec<Notification>) -> DiscordEmbedData {
        let embeds: Vec<DiscordEmbed> = notifications
            .into_iter()
            .map(|notification| {
                let fields = notification
                    .fields
                    .into_iter()
                    .map(|field| DiscordEmbedField {
                        name: field.name,
                        value: field.value,
                    })
                    .collect();

                DiscordEmbed {
                    title: notification.title,
                    fields,
                }
            })
            .collect();

        // Discordの制限に合わせて10個までに制限
        let limited_embeds = if embeds.len() > 10 {
            embeds[..10].to_vec()
        } else {
            embeds
        };

        DiscordEmbedData {
            embeds: limited_embeds,
        }
    }
}

impl<T: HttpClient + Send + Sync + 'static> NotificationService for DiscordNotificationService<T> {
    async fn send_notifications(&self, notifications: Vec<Notification>) -> Result<(), NotificationError> {
        // 通知をDiscord用のデータに変換
        let discord_data = self.to_discord_data(notifications);

        // Discordに送信
        self.http_client
            .post(&self.webhook_url, &discord_data)
            .await
            .map_err(|e| NotificationError::SendError(e.to_string()))
    }
}
