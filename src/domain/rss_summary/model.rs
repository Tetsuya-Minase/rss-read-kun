use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// RSSサマリーのレスポンスを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlesResponse {
    pub message: String,
    pub data: ArticlesData,
}

/// RSSサマリーのデータを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlesData {
    pub total: usize,
    pub summary: Vec<Category>,
}

/// カテゴリを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    #[serde(flatten)]
    pub category_map: HashMap<String, CategoryDetails>,
}

/// カテゴリの詳細を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDetails {
    #[serde(default)]
    pub category_count: Option<usize>,
    pub articles: Vec<Article>,
}

/// 記事を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub link: String,
}

impl ArticlesResponse {
    /// 新しいRSSサマリーレスポンスを作成する
    ///
    /// # Arguments
    /// * `message` - メッセージ
    /// * `data` - サマリーデータ
    pub fn new(message: String, data: ArticlesData) -> Self {
        Self { message, data }
    }
}

impl ArticlesData {
    /// 新しいRSSサマリーデータを作成する
    ///
    /// # Arguments
    /// * `total` - 記事の総数
    /// * `summary` - カテゴリのリスト
    pub fn new(total: usize, summary: Vec<Category>) -> Self {
        Self { total, summary }
    }

    /// 記事の総数を取得する
    pub fn total_articles(&self) -> usize {
        self.total
    }

    /// カテゴリの数を取得する
    pub fn category_count(&self) -> usize {
        self.summary.len()
    }
}

impl Category {
    /// カテゴリ名を取得する
    pub fn get_name(&self) -> Option<String> {
        self.category_map.keys().next().map(|s| s.clone())
    }

    /// カテゴリの詳細を取得する
    pub fn get_details(&self) -> Option<&CategoryDetails> {
        self.category_map.values().next()
    }
}

impl CategoryDetails {
    /// 記事の数を取得する
    pub fn article_count(&self) -> usize {
        self.articles.len()
    }
}
