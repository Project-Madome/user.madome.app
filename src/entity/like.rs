use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Sort;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LikeKind {
    Book,
    BookTag,
}

#[derive(Debug, Clone, Copy)]
pub enum LikeSortBy {
    CreatedAt(Sort),
    Random,
}

impl Default for LikeSortBy {
    fn default() -> Self {
        Self::CreatedAt(Sort::Desc)
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum Like {
    Book {
        book_id: u32,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
    BookTag {
        tag_kind: String, // artist or series or female or male or misc ...
        tag_name: String,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
}

impl Like {
    /// constructor of book
    pub fn book(user_id: Uuid, book_id: u32) -> Self {
        Self::Book {
            book_id,
            user_id,
            created_at: Utc::now(),
        }
    }

    /// constructor of book_tag
    pub fn book_tag(user_id: Uuid, tag_kind: String, tag_name: String) -> Self {
        Self::BookTag {
            user_id,
            tag_kind,
            tag_name,
            created_at: Utc::now(),
        }
    }

    pub fn kind(&self) -> LikeKind {
        match self {
            Like::Book { .. } => LikeKind::Book,
            Like::BookTag { .. } => LikeKind::BookTag,
        }
    }

    pub fn user_id(&self) -> Uuid {
        match self {
            Like::Book { user_id, .. } => *user_id,
            Like::BookTag { user_id, .. } => *user_id,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Like::Book { created_at, .. } => *created_at,
            Like::BookTag { created_at, .. } => *created_at,
        }
    }
}
