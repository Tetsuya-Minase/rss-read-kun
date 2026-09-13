use crate::application::use_case::fetch_and_summarize::AppError;
use crate::domain::notification::{Notification, NotificationField};

/// エラー情報から Discord 通知用の Notification を生成する
pub fn create_error_notification(error: &AppError) -> Notification {
    let (field_name, raw_message) = match error {
        AppError::RssError(msg) => ("RSSフィード取得エラー", msg.as_str()),
        AppError::SummaryError(msg) => ("要約生成エラー", msg.as_str()),
        AppError::NotificationError(msg) => ("通知送信エラー", msg.as_str()),
    };

    let field_value = if raw_message.is_empty() {
        "詳細情報なし".to_string()
    } else {
        let char_count = raw_message.chars().count();
        if char_count > 1024 {
            let truncated: String = raw_message.chars().take(1021).collect();
            format!("{}...", truncated)
        } else {
            raw_message.to_string()
        }
    };

    Notification {
        title: "⚠️ RSS処理でエラーが発生しました".to_string(),
        fields: vec![NotificationField {
            name: field_name.to_string(),
            value: field_value,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t1_rss_error() {
        let err = AppError::RssError("connection timeout".to_string());
        let notif = create_error_notification(&err);
        assert_eq!(notif.title, "⚠️ RSS処理でエラーが発生しました");
        assert_eq!(notif.fields.len(), 1);
        assert_eq!(notif.fields[0].name, "RSSフィード取得エラー");
        assert_eq!(notif.fields[0].value, "connection timeout");
    }

    #[test]
    fn test_t2_summary_error() {
        let err = AppError::SummaryError("Failed to extract summary from response".to_string());
        let notif = create_error_notification(&err);
        assert_eq!(notif.title, "⚠️ RSS処理でエラーが発生しました");
        assert_eq!(notif.fields.len(), 1);
        assert_eq!(notif.fields[0].name, "要約生成エラー");
        assert_eq!(notif.fields[0].value, "Failed to extract summary from response");
    }

    #[test]
    fn test_t3_notification_error() {
        let err = AppError::NotificationError("Status: 400, Body: invalid embed".to_string());
        let notif = create_error_notification(&err);
        assert_eq!(notif.title, "⚠️ RSS処理でエラーが発生しました");
        assert_eq!(notif.fields.len(), 1);
        assert_eq!(notif.fields[0].name, "通知送信エラー");
        assert_eq!(notif.fields[0].value, "Status: 400, Body: invalid embed");
    }

    #[test]
    fn test_t4_empty_message_sets_placeholder() {
        let err = AppError::RssError("".to_string());
        let notif = create_error_notification(&err);
        assert_eq!(notif.fields[0].value, "詳細情報なし");
    }

    #[test]
    fn test_t5_1024_chars_not_truncated() {
        let msg = "a".repeat(1024);
        let err = AppError::SummaryError(msg.clone());
        let notif = create_error_notification(&err);
        assert_eq!(notif.fields[0].value, msg);
        assert_eq!(notif.fields[0].value.chars().count(), 1024);
    }

    #[test]
    fn test_t6_1025_chars_truncated_to_1024() {
        let msg = "a".repeat(1025);
        let err = AppError::SummaryError(msg);
        let notif = create_error_notification(&err);
        let expected = format!("{}...", "a".repeat(1021));
        assert_eq!(notif.fields[0].value, expected);
        assert_eq!(notif.fields[0].value.chars().count(), 1024);
    }

    #[test]
    fn test_p1_invariants() {
        let errors = vec![
            AppError::RssError("a".to_string()),
            AppError::SummaryError("".to_string()),
            AppError::NotificationError("a".repeat(2000)),
            AppError::RssError("a".repeat(1024)),
            AppError::SummaryError("a".repeat(1025)),
            AppError::NotificationError("あ".repeat(2000)), // マルチバイト文字
        ];

        for err in errors {
            let notif = create_error_notification(&err);
            assert_eq!(notif.fields.len(), 1, "P1 invariant: fields.len() == 1");
            let val = &notif.fields[0].value;
            assert!(!val.is_empty(), "P1 invariant: !value.is_empty()");
            assert!(
                val.chars().count() <= 1024,
                "P1 invariant: value length <= 1024 chars"
            );
        }
    }
}
