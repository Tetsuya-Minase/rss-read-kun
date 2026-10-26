use crate::domain::model::rss_summary::{Article, ArticlesData, ArticlesResponse, Category, Highlight};
use crate::domain::notification::{Notification, NotificationField};

/// RSS要約レスポンスから通知のリストを生成する純粋関数
pub fn build_notifications(res: &ArticlesResponse) -> Vec<Notification> {
    let mut notifications = Vec::new();

    // ヘッダー
    let header_title = format!("📰 Zennトレンド {}件", res.data.total);
    let mut header_fields = Vec::new();
    
    for (i, highlight) in res.data.highlights.iter().enumerate() {
        header_fields.push(NotificationField {
            name: format!("{}. {}", i + 1, highlight.title),
            value: format!("{}\n[記事を読む]({})", highlight.summary, highlight.link),
        });
    }

    notifications.push(Notification {
        title: header_title,
        description: Some(res.message.clone()),
        fields: header_fields,
    });

    // カテゴリ
    for category in &res.data.categories {
        if category.articles.is_empty() {
            continue;
        }

        let category_title = format!("{} ({}件)", category.name, category.articles.len());
        
        let mut desc_lines = Vec::new();
        for article in &category.articles {
            let mut line = format!("・[{}]({})", article.title, article.link);
            if !article.summary.is_empty() {
                line.push_str(&format!(" — {}", article.summary));
            }
            desc_lines.push(line);
        }

        notifications.push(Notification {
            title: category_title,
            description: Some(desc_lines.join("\n")),
            fields: vec![],
        });
    }

    notifications
}

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用のヘルパー
    fn create_highlight(title: &str, summary: &str, link: &str) -> Highlight {
        Highlight {
            title: title.to_string(),
            summary: summary.to_string(),
            link: link.to_string(),
        }
    }

    fn create_article(title: &str, summary: &str, link: &str) -> Article {
        Article {
            title: title.to_string(),
            summary: summary.to_string(),
            link: link.to_string(),
        }
    }

    // T1: 正常系 ヘッダー通知にタイトルと全体メッセージが入る
    #[test]
    fn test_t1_header_notification() {
        let res = ArticlesResponse {
            message: "AI関連の話題が中心の1日。".to_string(),
            data: ArticlesData {
                total: 19,
                highlights: vec![create_highlight("1年半かけて育ててきたAI開発フロー", "Claude Code中心の開発フローを半年単位の試行錯誤込みで公開。", "https://zenn.dev/a1")],
                categories: vec![Category {
                    name: "🤖 AI・LLM".to_string(),
                    articles: vec![create_article("ローカルJevの可能性検証", "ローカル実行のJevを実機検証", "https://zenn.dev/a2")],
                }],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result[0].title, "📰 Zennトレンド 19件");
        assert_eq!(result[0].description, Some("AI関連の話題が中心の1日。".to_string()));
    }

    // T2: 正常系 ハイライトが順位付きフィールドになる
    #[test]
    fn test_t2_highlights_as_ranked_fields() {
        let res = ArticlesResponse {
            message: "AI関連の話題が中心の1日。".to_string(),
            data: ArticlesData {
                total: 19,
                highlights: vec![create_highlight("1年半かけて育ててきたAI開発フロー", "Claude Code中心の開発フローを半年単位の試行錯誤込みで公開。", "https://zenn.dev/a1")],
                categories: vec![Category {
                    name: "🤖 AI・LLM".to_string(),
                    articles: vec![create_article("ローカルJevの可能性検証", "ローカル実行のJevを実機検証", "https://zenn.dev/a2")],
                }],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result[0].fields.len(), 1);
        assert_eq!(result[0].fields[0].name, "1. 1年半かけて育ててきたAI開発フロー");
        assert_eq!(result[0].fields[0].value, "Claude Code中心の開発フローを半年単位の試行錯誤込みで公開。\n[記事を読む](https://zenn.dev/a1)");
    }

    // T3: 正常系 ハイライト3件が入力順に1/2/3と採番される
    #[test]
    fn test_t3_highlights_ordering() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 3,
                highlights: vec![
                    create_highlight("A", "要約", "https://zenn.dev/a"),
                    create_highlight("B", "要約", "https://zenn.dev/b"),
                    create_highlight("C", "要約", "https://zenn.dev/c"),
                ],
                categories: vec![],
            },
        };

        let result = build_notifications(&res);
        let names: Vec<String> = result[0].fields.iter().map(|f| f.name.clone()).collect();
        assert_eq!(names, vec!["1. A", "2. B", "3. C"]);
    }

    // T4: 正常系 カテゴリ通知のタイトルに記事件数が入る
    #[test]
    fn test_t4_category_title_contains_count() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 2,
                highlights: vec![],
                categories: vec![Category {
                    name: "🤖 AI・LLM".to_string(),
                    articles: vec![
                        create_article("A1", "S1", "L1"),
                        create_article("A2", "S2", "L2"),
                    ],
                }],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result[1].title, "🤖 AI・LLM (2件)");
    }

    // T5: 正常系 カテゴリ通知が1行リンク形式の description になり fields は空
    #[test]
    fn test_t5_category_description_format() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 2,
                highlights: vec![],
                categories: vec![Category {
                    name: "🤖 AI・LLM".to_string(),
                    articles: vec![
                        create_article("ローカルJevの可能性検証", "ローカル実行のJevを実機検証", "https://zenn.dev/a2"),
                        create_article("認知負債の未来は？", "技術的負債をAI時代に読み替える論考", "https://zenn.dev/a3"),
                    ],
                }],
            },
        };

        let result = build_notifications(&res);
        let expected_desc = "・[ローカルJevの可能性検証](https://zenn.dev/a2) — ローカル実行のJevを実機検証\n・[認知負債の未来は？](https://zenn.dev/a3) — 技術的負債をAI時代に読み替える論考";
        assert_eq!(result[1].description, Some(expected_desc.to_string()));
        assert!(result[1].fields.is_empty());
    }

    // T6: 境界値 記事0件のカテゴリは embed を生成しない
    #[test]
    fn test_t6_empty_category_is_skipped() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![
                    Category { name: "🤖 AI・LLM".to_string(), articles: vec![create_article("A1", "S1", "L1")] },
                    Category { name: "🎮 ゲーム開発".to_string(), articles: vec![] },
                ],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result.len(), 2);
        assert!(!result.iter().any(|n| n.title.starts_with("🎮 ゲーム開発")));
    }

    // T7: 正常系 カテゴリの並び順が入力順のまま保たれる
    #[test]
    fn test_t7_category_ordering() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 2,
                highlights: vec![],
                categories: vec![
                    Category { name: "🎮 ゲーム開発".to_string(), articles: vec![create_article("A1", "S1", "L1")] },
                    Category { name: "🤖 AI・LLM".to_string(), articles: vec![create_article("A2", "S2", "L2")] },
                ],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result[1].title, "🎮 ゲーム開発 (1件)");
        assert_eq!(result[2].title, "🤖 AI・LLM (1件)");
    }

    // T8: 異常系 ハイライト0件でもヘッダー通知は生成される
    #[test]
    fn test_t8_header_without_highlights() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![Category { name: "🤖 AI・LLM".to_string(), articles: vec![create_article("A1", "S1", "L1")] }],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "📰 Zennトレンド 1件");
        assert!(result[0].fields.is_empty());
    }

    // T9: 境界値 記事が1件もない場合はヘッダー通知のみ
    #[test]
    fn test_t9_no_articles_at_all() {
        let res = ArticlesResponse {
            message: "対象記事はありませんでした。".to_string(),
            data: ArticlesData { total: 0, highlights: vec![], categories: vec![] },
        };

        let result = build_notifications(&res);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "📰 Zennトレンド 0件");
        assert!(result[0].fields.is_empty());
    }

    // T10: 境界値 要約が空文字の記事は区切り記号を出さない
    #[test]
    fn test_t10_empty_summary_article() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![Category { name: "📚 その他".to_string(), articles: vec![create_article("X", "", "https://zenn.dev/x")] }],
            },
        };

        let result = build_notifications(&res);
        assert_eq!(result[1].description, Some("・[X](https://zenn.dev/x)".to_string()));
    }

    // P2: 不変条件 result.len() == 1 + categories.iter().filter(|c| !c.articles.is_empty()).count()
    #[test]
    fn test_p2_result_length_invariant() {
        let inputs = vec![
            ArticlesResponse {
                message: "m".to_string(),
                data: ArticlesData { total: 0, highlights: vec![], categories: vec![] },
            },
            ArticlesResponse {
                message: "m".to_string(),
                data: ArticlesData { total: 1, highlights: vec![create_highlight("H", "S", "L")], categories: vec![] },
            },
            ArticlesResponse {
                message: "m".to_string(),
                data: ArticlesData {
                    total: 1, highlights: vec![], categories: vec![
                        Category { name: "C1".to_string(), articles: vec![] },
                        Category { name: "C2".to_string(), articles: vec![create_article("A", "S", "L")] },
                    ],
                },
            },
        ];

        for res in inputs {
            let result = build_notifications(&res);
            let expected_len = 1 + res.data.categories.iter().filter(|c| !c.articles.is_empty()).count();
            assert_eq!(result.len(), expected_len);
        }
    }
}
