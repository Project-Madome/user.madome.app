use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Sort;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HistoryKind {
    Book,
}

#[derive(Debug, Clone, Copy)]
pub enum HistorySortBy {
    CreatedAt(Sort),
    UpdatedAt(Sort),
    Random,
}

#[derive(Debug)]
pub enum History {
    Book {
        book_id: u32,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
}

impl History {
    /// constructor of history
    pub fn book(book_id: u32, user_id: Uuid) -> Self {
        let now = Utc::now();

        Self::Book {
            book_id,
            user_id,
            created_at: now,
            updated_at: now,
        }
    }

    /// kind of history
    pub fn kind(&self) -> HistoryKind {
        match self {
            Self::Book { .. } => HistoryKind::Book,
        }
    }

    /// history.user_id
    pub fn user_id(&self) -> Uuid {
        match self {
            Self::Book { user_id, .. } => *user_id,
        }
    }

    /// history.created_at
    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::Book { created_at, .. } => *created_at,
        }
    }
}
