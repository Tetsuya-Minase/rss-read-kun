use crate::domain::model::rss_summary::ArticlesResponse;
use crate::domain::notification::Notification;
use crate::application::notification::summary_renderer::build_notifications;

pub const SKIP_NOTE_RESERVE: usize = 64;
pub const MAX_EMBEDS: usize = 10;
pub const MAX_EMBED_TITLE_LEN: usize = 256;
pub const MAX_EMBED_DESC_LEN: usize = 4096;
pub const MAX_FIELD_NAME_LEN: usize = 256;
pub const MAX_FIELD_VALUE_LEN: usize = 1024;
pub const MAX_FIELDS: usize = 25;
pub const MAX_TOTAL_LEN: usize = 6000;

#[derive(Debug)]
pub struct ValidatedNotifications(pub Vec<Notification>);

#[derive(Debug)]
pub struct DigestPlan {
    pub notifications: ValidatedNotifications,
    pub skipped: usize,
}

pub fn plan_digest(res: &ArticlesResponse) -> DigestPlan {
    let mut current_notifications = build_notifications(&ArticlesResponse {
        message: res.message.clone(),
        data: crate::domain::model::rss_summary::ArticlesData {
            total: res.data.total,
            highlights: vec![],
            categories: vec![],
        }
    });
    
    let mut skipped = 0;

    // Highlights
    let mut valid_highlights = vec![];
    for highlight in &res.data.highlights {
        valid_highlights.push(highlight.clone());
        let candidate_res = ArticlesResponse {
            message: res.message.clone(),
            data: crate::domain::model::rss_summary::ArticlesData {
                total: res.data.total,
                highlights: valid_highlights.clone(),
                categories: vec![],
            }
        };
        let candidate = build_notifications(&candidate_res);
        if fits(&candidate) {
            current_notifications = candidate;
        } else {
            valid_highlights.pop();
            skipped += 1;
        }
    }

    // Categories
    let mut valid_categories: Vec<crate::domain::model::rss_summary::Category> = vec![];
    for category in &res.data.categories {
        let mut valid_articles = vec![];
        for article in &category.articles {
            valid_articles.push(article.clone());
            
            let mut test_categories = valid_categories.clone();
            test_categories.push(crate::domain::model::rss_summary::Category {
                name: category.name.clone(),
                articles: valid_articles.clone(),
            });
            
            let candidate_res = ArticlesResponse {
                message: res.message.clone(),
                data: crate::domain::model::rss_summary::ArticlesData {
                    total: res.data.total,
                    highlights: valid_highlights.clone(),
                    categories: test_categories.clone(),
                }
            };
            
            let candidate = build_notifications(&candidate_res);
            if fits(&candidate) {
                current_notifications = candidate;
            } else {
                valid_articles.pop();
                skipped += 1;
            }
        }
        
        if !valid_articles.is_empty() {
            valid_categories.push(crate::domain::model::rss_summary::Category {
                name: category.name.clone(),
                articles: valid_articles,
            });
        }
    }

    if skipped > 0 {
        if let Some(n) = current_notifications.get_mut(0) {
            let warn_text = format!("⚠️ Discordの制限により{}件を除外しました", skipped);
            if let Some(desc) = &mut n.description {
                desc.push_str(&format!("\n{}", warn_text));
            } else {
                n.description = Some(warn_text);
            }
        }
    }

    DigestPlan {
        notifications: ValidatedNotifications(current_notifications),
        skipped,
    }
}

fn fits(notifications: &[Notification]) -> bool {
    if notifications.len() > MAX_EMBEDS {
        return false;
    }
    
    let mut total_chars = 0;
    
    for n in notifications {
        let title_len = n.title.chars().count();
        if title_len > MAX_EMBED_TITLE_LEN {
            return false;
        }
        total_chars += title_len;
        
        if let Some(desc) = &n.description {
            let desc_len = desc.chars().count();
            if desc_len > MAX_EMBED_DESC_LEN {
                return false;
            }
            total_chars += desc_len;
        }
        
        if n.fields.len() > MAX_FIELDS {
            return false;
        }
        
        for f in &n.fields {
            let name_len = f.name.chars().count();
            let val_len = f.value.chars().count();
            if name_len > MAX_FIELD_NAME_LEN || val_len > MAX_FIELD_VALUE_LEN {
                return false;
            }
            total_chars += name_len + val_len;
        }
    }
    
    if total_chars > MAX_TOTAL_LEN - SKIP_NOTE_RESERVE {
        return false;
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::rss_summary::{Article, ArticlesData, Category, Highlight};
    use crate::domain::notification::NotificationField;

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

    // T39: 正常系 制限内の入力は全件通過し除外0件
    #[test]
    fn test_t39_within_limits_no_skip() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![Category {
                    name: "Tech".to_string(),
                    articles: vec![create_article("A", "S", "L")],
                }],
            },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 0);
        assert_eq!(plan.notifications.0.len(), 2); // Header + 1 Category
    }

    // T40: 境界値 field.value が1024文字ちょうどなら通過する
    #[test]
    fn test_t40_field_value_exact_1024() {
        // 固定部 "\n[記事を読む]()" is 10 + link len (18) = 28 characters
        // So summary needs to be 996 chars. 996 + 28 = 1024
        let summary_text = "あ".repeat(996);
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![create_highlight("H", &summary_text, "https://zenn.dev/h")],
                categories: vec![],
            },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 0);
        let val_len = plan.notifications.0[0].fields[0].value.chars().count();
        assert_eq!(val_len, 1024);
    }

    // T41: 境界値 field.value が1025文字になる記事だけ除外され、採番は詰められる
    #[test]
    fn test_t41_field_value_exceeds_1024() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 2,
                highlights: vec![
                    create_highlight("NG", &"あ".repeat(997), "https://zenn.dev/h"), // 997 + 28 = 1025 -> skipped
                    create_highlight("OK", "短い要約", "https://zenn.dev/o"),
                ],
                categories: vec![],
            },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 1);
        assert_eq!(plan.notifications.0[0].fields.len(), 1);
        assert_eq!(plan.notifications.0[0].fields[0].name, "1. OK");
    }

    // T42: 境界値 カテゴリ名が長すぎて title が256文字を超える場合はカテゴリごと除外
    #[test]
    fn test_t42_category_title_exceeds_256() {
        // " (2件)" is 5 chars. Name needs to be 252 chars.
        let name = "あ".repeat(252); 
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 2,
                highlights: vec![],
                categories: vec![Category {
                    name,
                    articles: vec![create_article("A1", "S1", "L1"), create_article("A2", "S2", "L2")],
                }],
            },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 2);
        assert_eq!(plan.notifications.0.len(), 1); // Only header
    }

    // T43: 境界値 カテゴリ description が4096文字を超える場合は末尾記事から除外
    #[test]
    fn test_t43_category_description_exceeds_4096() {
        // 1行 = ・[A](L) — S \n = 1 + 10 + 19 + 3 + 373 = 406 + 4(markup) = 410. Wait, exact calculation doesn't matter, just large enough
        let mut articles = Vec::new();
        for i in 0..10 {
            articles.push(create_article(&"あ".repeat(10), &"い".repeat(373), &format!("https://zenn.dev/a{}", i)));
        }
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 10,
                highlights: vec![],
                categories: vec![Category {
                    name: "📚 その他".to_string(),
                    articles,
                }],
            },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 1);
        let desc = plan.notifications.0[1].description.as_ref().unwrap();
        assert!(!desc.contains("https://zenn.dev/a9"));
    }

    // T44: 境界値 fields が26件になる場合は25件に収まり1件が除外される
    #[test]
    fn test_t44_fields_exceeds_25() {
        let mut highlights = Vec::new();
        for i in 0..26 {
            highlights.push(create_highlight(&format!("H{}", i), "s", &format!("https://zenn.dev/h{}", i)));
        }
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 26,
                highlights,
                categories: vec![],
            },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 1);
        assert_eq!(plan.notifications.0[0].fields.len(), 25);
    }

    // T45: 境界値 カテゴリ11個は embed 10個に収まり超過分が除外件数に計上される
    #[test]
    fn test_t45_embeds_exceeds_10() {
        let mut categories = Vec::new();
        for i in 0..11 {
            categories.push(Category {
                name: format!("C{}", i),
                articles: vec![create_article("A", "S", "L")],
            });
        }
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData { total: 11, highlights: vec![], categories },
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 2);
        assert_eq!(plan.notifications.0.len(), 10);
    }

    // T46: 境界値 合計文字数は実効予算5936文字以下に収まる
    #[test]
    fn test_t46_total_chars_exceeds_budget() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 2,
                highlights: vec![],
                categories: vec![
                    Category { name: "C1".to_string(), articles: vec![create_article("A", &"s".repeat(3000), "L")] },
                    Category { name: "C2".to_string(), articles: vec![create_article("B", &"s".repeat(3000), "L")] },
                ]
            }
        };
        let plan = plan_digest(&res);
        assert!(plan.skipped >= 1);
        let total_chars: usize = plan.notifications.0.iter().map(|n| {
            n.title.chars().count() + 
            n.description.as_ref().map(|d| d.chars().count()).unwrap_or(0) +
            n.fields.iter().map(|f| f.name.chars().count() + f.value.chars().count()).sum::<usize>()
        }).sum();
        assert!(total_chars <= 5936);
    }

    // T47: 正常系 除外発生時にヘッダーへ件数付き警告が付記される
    #[test]
    fn test_t47_skipped_warning_added_to_header() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 11,
                highlights: vec![],
                categories: (0..11).map(|i| Category {
                    name: format!("C{}", i),
                    articles: vec![create_article("A", "S", "L")],
                }).collect()
            }
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.notifications.0[0].description, Some("m\n⚠️ Discordの制限により2件を除外しました".to_string()));
    }

    // T48: 正常系 除外0件なら警告は付かない
    #[test]
    fn test_t48_no_skipped_warning() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![Category { name: "C".to_string(), articles: vec![create_article("A", "S", "L")] }]
            }
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.notifications.0[0].description, Some("m".to_string()));
    }

    // T49: 境界値 カテゴリの全記事が除外されたら embed を作らない
    #[test]
    fn test_t49_all_articles_skipped_removes_category() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 1,
                highlights: vec![],
                categories: vec![Category { name: "C".to_string(), articles: vec![create_article("A", &"s".repeat(5000), "L")] }]
            }
        };
        let plan = plan_digest(&res);
        assert_eq!(plan.skipped, 1);
        assert_eq!(plan.notifications.0.len(), 1); // Header only
    }

    // P1, P3, P4, P5, P6 are implicitly tested through boundary tests above, but let's add P1 and P6 checks
    #[test]
    fn test_p1_p6_invariants() {
        let res = ArticlesResponse {
            message: "m".to_string(),
            data: ArticlesData {
                total: 3,
                highlights: vec![create_highlight("H1", &"s".repeat(1500), "L1")], // Exceeds field value -> skipped
                categories: vec![
                    Category { name: "C1".to_string(), articles: vec![create_article("A1", "S1", "L2"), create_article("A2", "S2", "L3")] }
                ]
            }
        };
        let input_articles = 3;
        let plan = plan_digest(&res);
        
        // P6: 採用 + skipped = 入力記事総数
        let adopted: usize = plan.notifications.0[0].fields.len() + 
                             if plan.notifications.0.len() > 1 { plan.notifications.0[1].description.as_ref().unwrap().matches("・[").count() } else { 0 };
        assert_eq!(adopted + plan.skipped, input_articles);
        
        // P1: Link appears exactly once if adopted, 0 if skipped
        let json = serde_json::to_string(&plan.notifications.0.iter().map(|n| n.title.clone()).collect::<Vec<_>>()).unwrap() + 
                   &serde_json::to_string(&plan.notifications.0.iter().map(|n| n.description.clone()).collect::<Vec<_>>()).unwrap() +
                   &serde_json::to_string(&plan.notifications.0.iter().flat_map(|n| n.fields.clone()).map(|f| f.value).collect::<Vec<_>>()).unwrap();
        assert_eq!(json.matches("L1").count(), 0);
        assert_eq!(json.matches("L2").count(), 1);
        assert_eq!(json.matches("L3").count(), 1);
    }
}
