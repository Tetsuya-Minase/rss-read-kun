use serde::{Deserialize, Serialize};

/// RSSデータを表す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RssData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
}

impl RssData {
    /// 新しいRSSデータを作成する
    ///
    /// # Arguments
    /// * `title` - タイトル
    /// * `description` - 説明
    /// * `link` - リンク
    pub fn new(
        title: Option<String>,
        description: Option<String>,
        link: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            link,
        }
    }

    /// RSSデータが有効かどうかを確認する
    pub fn is_valid(&self) -> bool {
        self.title.is_some() || self.description.is_some() || self.link.is_some()
    }
}
