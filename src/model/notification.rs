use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::Serialize;

use crate::entity;

use super::Presenter;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Notification {
    Book {
        book_id: u32,
        book_tags: Vec<(String, String)>,
        created_at: DateTime<Utc>,
    },
}

impl Presenter for Vec<Notification> {
    fn to_http(self, response: hyper::http::response::Builder) -> hyper::Response<hyper::Body> {
        let serialized = serde_json::to_string(&self).expect("serialize json");

        response
            .status(StatusCode::OK)
            .body(serialized.into())
            .unwrap()
    }
}

impl From<entity::Notification> for Notification {
    fn from(notification: entity::Notification) -> Self {
        pub use entity::Notification::*;

        match notification {
            Book {
                book_id,
                book_tags,
                created_at,
                ..
            } => Notification::Book {
                book_id,
                book_tags,
                created_at,
            },
        }
    }
}
