use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::Sort;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DislikeKind {
    Book,
    BookTag,
}

#[derive(Debug, Clone, Copy)]
pub enum DislikeSortBy {
    CreatedAt(Sort),
    Random,
}

impl Default for DislikeSortBy {
    fn default() -> Self {
        Self::CreatedAt(Sort::Desc)
    }
}

#[derive(Debug, Clone)]
pub enum Dislike {
    Book {
        book_id: u32,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
}

impl Dislike {
    /// constructor of Like::Book
    pub fn book(user_id: Uuid, book_id: u32) -> Self {
        Self::Book {
            book_id,
            user_id,
            created_at: Utc::now(),
        }
    }

    /// constructor of Like::BookTag
    pub fn book_tag(user_id: Uuid, tag_kind: String, tag_name: String) -> Self {
        Self::BookTag {
            user_id,
            tag_kind,
            tag_name,
            created_at: Utc::now(),
        }
    }

    pub fn kind(&self) -> DislikeKind {
        match self {
            Dislike::Book { .. } => DislikeKind::Book,
            Dislike::BookTag { .. } => DislikeKind::BookTag,
        }
    }

    pub fn user_id(&self) -> Uuid {
        match self {
            Dislike::Book { user_id, .. } => *user_id,
            Dislike::BookTag { user_id, .. } => *user_id,
        }
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        match self {
            Dislike::Book { created_at, .. } => *created_at,
            Dislike::BookTag { created_at, .. } => *created_at,
        }
    }
}
