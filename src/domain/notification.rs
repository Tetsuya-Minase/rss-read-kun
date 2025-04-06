use std::error::Error;
use std::fmt;

/// 通知関連のエラー型
#[derive(Debug)]
pub enum NotificationError {
    SendError(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationError::SendError(e) => write!(f, "Notification send error: {}", e),
        }
    }
}

impl Error for NotificationError {}

/// 通知フィールドを表す構造体
#[derive(Debug, Clone)]
pub struct NotificationField {
    pub name: String,
    pub value: String,
}

/// 通知を表す構造体
#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub fields: Vec<NotificationField>,
}

/// 通知サービスのトレイト
pub trait NotificationService {
    /// 通知を送信する
    ///
    /// # Arguments
    /// * `notifications` - 送信する通知のリスト
    async fn send_notifications(&self, notifications: Vec<Notification>) -> Result<(), NotificationError>;
}
