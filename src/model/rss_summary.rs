use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticlesResponse {
    pub message: String,
    pub data: ArticlesData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticlesData {
    pub total: usize,
    pub summary: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    #[serde(flatten)]
    pub category_map: HashMap<String, CategoryDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryDetails {
    #[serde(default)]
    pub category_count: Option<usize>,
    pub articles: Vec<Article>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub link: String,
}
