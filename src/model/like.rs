use chrono::{DateTime, Utc};
use hyper::{header, StatusCode};
use serde::Serialize;
use uuid::Uuid;

use crate::entity;

use super::Presenter;

/* #[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")] */
pub enum Like {
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

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LikeWithoutUserId {
    Book {
        book_id: u32,
        created_at: DateTime<Utc>,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        created_at: DateTime<Utc>,
    },
}

impl Presenter for Vec<Like> {
    fn to_http(self, response: hyper::http::response::Builder) -> hyper::Response<hyper::Body> {
        let like: Vec<LikeWithoutUserId> = self.into_iter().map(Into::into).collect();
        let serialized = serde_json::to_string(&like).expect("json serialize");

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
    }
}

impl Presenter for Like {
    fn to_http(self, response: hyper::http::response::Builder) -> hyper::Response<hyper::Body> {
        let like: LikeWithoutUserId = self.into();
        let serialized = serde_json::to_string(&like).expect("json serialize");

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
    }
}

impl From<Like> for LikeWithoutUserId {
    fn from(like: Like) -> Self {
        match like {
            Like::Book {
                book_id,
                created_at,
                ..
            } => Self::Book {
                book_id,
                created_at,
            },
            Like::BookTag {
                tag_kind,
                tag_name,
                created_at,
                ..
            } => Self::BookTag {
                tag_kind,
                tag_name,
                created_at,
            },
        }
    }
}

impl From<entity::Like> for Like {
    fn from(like: entity::Like) -> Self {
        match like {
            entity::Like::Book {
                book_id,
                user_id,
                created_at,
            } => Self::Book {
                book_id,
                user_id,
                created_at,
            },
            entity::Like::BookTag {
                tag_kind,
                tag_name,
                user_id,
                created_at,
            } => Self::BookTag {
                tag_kind,
                tag_name,
                user_id,
                created_at,
            },
        }
    }
}
