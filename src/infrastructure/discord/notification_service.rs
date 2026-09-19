use serde::Serialize;
use crate::domain::notification::{Notification, NotificationError, NotificationService};
use crate::infrastructure::http_client::HttpClient;

use crate::application::notification::discord_limits::ValidatedNotifications;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
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
    pub fn new(http_client: T, webhook_url: String) -> Self {
        Self {
            http_client,
            webhook_url,
        }
    }

    fn to_discord_data(&self, notifications: ValidatedNotifications) -> DiscordEmbedData {
        let embeds: Vec<DiscordEmbed> = notifications.0
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
                    description: notification.description,
                    fields,
                }
            })
            .collect();

        DiscordEmbedData { embeds }
    }
}

impl<T: HttpClient + Send + Sync + 'static> NotificationService for DiscordNotificationService<T> {
    async fn send_notifications(&self, notifications: ValidatedNotifications) -> Result<(), NotificationError> {
        let discord_data = self.to_discord_data(notifications);
        self.http_client
            .post(&self.webhook_url, &discord_data)
            .await
            .map_err(|e| NotificationError::SendError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::notification::NotificationField;

    struct DummyHttpClient;
    impl HttpClient for DummyHttpClient {
        fn get(&self, _url: &str) -> impl std::future::Future<Output = Result<rss::Channel, crate::infrastructure::http_client::HttpClientError>> + Send {
            async { unimplemented!() }
        }

        fn post<T: Serialize + ?Sized + Send + Sync>(
            &self,
            _url: &str,
            _body: &T,
        ) -> impl std::future::Future<Output = Result<(), crate::infrastructure::http_client::HttpClientError>> + Send {
            async { Ok(()) }
        }

        fn post_with_response<T: Serialize + ?Sized + Send + Sync, R: for<'de> serde::Deserialize<'de> + Send>(
            &self,
            _url: &str,
            _body: &T,
        ) -> impl std::future::Future<Output = Result<R, crate::infrastructure::http_client::HttpClientError>> + Send {
            async { unimplemented!() }
        }

        fn post_with_response_and_headers<T: Serialize + ?Sized + Send + Sync, R: for<'de> serde::Deserialize<'de> + Send>(
            &self,
            _url: &str,
            _headers: Vec<(String, String)>,
            _body: &T,
        ) -> impl std::future::Future<Output = Result<R, crate::infrastructure::http_client::HttpClientError>> + Send {
            async { unimplemented!() }
        }
    }

    // T50: 正常系 検証済みデータを一切改変せず直列化する
    // P7: to_discord_data の恒等性
    #[test]
    fn test_t50_p7_no_truncation_or_modification() {
        let service = DiscordNotificationService::new(DummyHttpClient, "url".to_string());
        
        let notifs = ValidatedNotifications(vec![
            Notification {
                title: "T1".to_string(),
                description: Some("D1".to_string()),
                fields: vec![
                    NotificationField { name: "N1".to_string(), value: "V1".to_string() }
                ]
            },
            Notification {
                title: "T2".to_string(),
                description: None,
                fields: vec![]
            }
        ]);
        
        let data = service.to_discord_data(notifs);
        
        assert_eq!(data.embeds.len(), 2);
        
        assert_eq!(data.embeds[0].title, "T1");
        assert_eq!(data.embeds[0].description.as_deref(), Some("D1"));
        assert_eq!(data.embeds[0].fields.len(), 1);
        assert_eq!(data.embeds[0].fields[0].name, "N1");
        assert_eq!(data.embeds[0].fields[0].value, "V1");
        
        assert_eq!(data.embeds[1].title, "T2");
        assert_eq!(data.embeds[1].description, None);
        assert!(data.embeds[1].fields.is_empty());
        
        // Ensure no "..." is added anywhere (modification check)
        let json = serde_json::to_string(&data).unwrap();
        assert!(!json.contains("..."));
    }

    #[test]
    fn test_t11_description_some_includes_key() {
        let service = DiscordNotificationService::new(DummyHttpClient, "url".to_string());
        let notifs = ValidatedNotifications(vec![crate::domain::notification::Notification {
            title: "T".to_string(),
            description: Some("desc".to_string()),
            fields: vec![],
        }]);
        let data = service.to_discord_data(notifs);
        let json = serde_json::to_value(&data).unwrap();
        assert!(json["embeds"][0].as_object().unwrap().contains_key("description"));
    }

    #[test]
    fn test_t12_description_none_skips_key() {
        let service = DiscordNotificationService::new(DummyHttpClient, "url".to_string());
        let notifs = ValidatedNotifications(vec![crate::domain::notification::Notification {
            title: "T".to_string(),
            description: None,
            fields: vec![],
        }]);
        let data = service.to_discord_data(notifs);
        let json = serde_json::to_value(&data).unwrap();
        assert!(!json["embeds"][0].as_object().unwrap().contains_key("description"));
    }

    #[test]
    fn test_t22_plan_digest_to_discord_data_integration() {
        use crate::domain::model::rss_summary::{Article, ArticlesData, ArticlesResponse, Category, Highlight};
        use crate::application::notification::discord_limits::plan_digest;

        let mut highlights = Vec::new();
        for i in 0..5 {
            highlights.push(Highlight {
                title: format!("H{}", i),
                summary: format!("Summary H{}", i),
                link: format!("https://zenn.dev/h{}", i),
            });
        }

        let mut categories = Vec::new();
        for i in 0..7 {
            let mut articles = Vec::new();
            for j in 0..2 {
                articles.push(Article {
                    title: format!("A{}{}", i, j),
                    summary: format!("Summary A{}{}", i, j),
                    link: format!("https://zenn.dev/a{}{}", i, j),
                });
            }
            categories.push(Category {
                name: format!("Category {}", i),
                articles,
            });
        }

        let res = ArticlesResponse {
            message: "Success".to_string(),
            data: ArticlesData {
                total: 19,
                highlights,
                categories,
            },
        };

        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 0);

        let service = DiscordNotificationService::new(DummyHttpClient, "url".to_string());
        let data = service.to_discord_data(plan.notifications);
        
        assert_eq!(data.embeds.len(), 8);

        let json = serde_json::to_value(&data).unwrap();
        let json_str = serde_json::to_string(&json).unwrap();

        let link_count = json_str.matches("https://zenn.dev/").count();
        assert_eq!(link_count, 19);
    }
}
