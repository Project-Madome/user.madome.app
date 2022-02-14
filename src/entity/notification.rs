use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Sort;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationKind {
    Book,
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationSortBy {
    CreatedAt(Sort),
}

#[derive(Debug, Clone)]
pub enum Notification {
    Book {
        book_id: u32,
        /// 알림의 이유
        book_tags: Vec<(String, String)>,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
}

impl Notification {
    pub fn book(user_id: Uuid, book_id: u32, book_tags: Vec<(String, String)>) -> Self {
        Self::Book {
            user_id,
            book_id,
            book_tags,
            created_at: Utc::now(),
        }
    }

    pub fn kind(&self) -> NotificationKind {
        match self {
            Self::Book { .. } => NotificationKind::Book,
        }
    }

    pub fn user_id(&self) -> Uuid {
        match self {
            Self::Book { user_id, .. } => *user_id,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::Book { created_at, .. } => *created_at,
        }
    }
}
