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
    #[serde(default)]
    pub highlights: Vec<Highlight>,
    #[serde(default)]
    pub categories: Vec<Category>,
}

/// ハイライト記事を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub title: String,
    pub summary: String,
    pub link: String,
}

/// カテゴリを表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub articles: Vec<Article>,
}

/// 記事を表す構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub summary: String,
    pub link: String,
}

impl ArticlesResponse {
    /// 新しいRSSサマリーレスポンスを作成する
    pub fn new(message: String, data: ArticlesData) -> Self {
        Self { message, data }
    }
}

impl ArticlesData {
    /// 新しいRSSサマリーデータを作成する
    pub fn new(total: usize, highlights: Vec<Highlight>, categories: Vec<Category>) -> Self {
        Self { total, highlights, categories }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Given: 新しい形式のJSON文字列
    // When: serde_json::from_str を呼び出す
    // Then: 正しく ArticlesResponse にデシリアライズされる
    #[test]
    fn test_t28_deserialize_new_format() {
        let json = r#"{"message":"m","data":{"total":2,"highlights":[{"title":"h","summary":"hs","link":"https://zenn.dev/h"}],"categories":[{"name":"🤖 AI・LLM","articles":[{"title":"a","summary":"as","link":"https://zenn.dev/a"}]}]}}"#;
        
        let result = serde_json::from_str::<ArticlesResponse>(json);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let r = result.unwrap();
        assert_eq!(r.data.total, 2);
        assert_eq!(r.data.highlights.len(), 1);
        assert_eq!(r.data.highlights[0].link, "https://zenn.dev/h");
        assert_eq!(r.data.categories.len(), 1);
        assert_eq!(r.data.categories[0].name, "🤖 AI・LLM");
        assert_eq!(r.data.categories[0].articles.len(), 1);
        assert_eq!(r.data.categories[0].articles[0].summary, "as");
    }

    // Given: highlightsキーが欠落したJSON文字列
    // When: serde_json::from_str を呼び出す
    // Then: highlightsが空のVecとしてデシリアライズされる
    #[test]
    fn test_t29_deserialize_missing_highlights() {
        let json = r#"{"message":"m","data":{"total":0,"categories":[]}}"#;
        let result = serde_json::from_str::<ArticlesResponse>(json);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let r = result.unwrap();
        assert!(r.data.highlights.is_empty());
    }

    // Given: categoriesキーが欠落したJSON文字列
    // When: serde_json::from_str を呼び出す
    // Then: categoriesが空のVecとしてデシリアライズされる
    #[test]
    fn test_t30_deserialize_missing_categories() {
        let json = r#"{"message":"m","data":{"total":0,"highlights":[]}}"#;
        let result = serde_json::from_str::<ArticlesResponse>(json);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let r = result.unwrap();
        assert!(r.data.categories.is_empty());
    }

    // Given: enum外のカテゴリ名を含むJSON文字列
    // When: serde_json::from_str を呼び出す
    // Then: パースエラーにならずにカテゴリ名が保持される
    #[test]
    fn test_t31_deserialize_unknown_category() {
        let json = r#"{"message":"m","data":{"total":1,"highlights":[],"categories":[{"name":"🧪 未知カテゴリ","articles":[{"title":"a","summary":"s","link":"https://zenn.dev/a"}]}]}}"#;
        let result = serde_json::from_str::<ArticlesResponse>(json);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let r = result.unwrap();
        assert_eq!(r.data.categories.len(), 1);
        assert_eq!(r.data.categories[0].name, "🧪 未知カテゴリ");
    }
}
