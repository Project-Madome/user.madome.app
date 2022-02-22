use std::sync::Arc;

use chrono::{DateTime, Utc};
use hyper::{header, Body, Request, Response, StatusCode};
use serde::Serialize;
use util::http::SetResponse;

use crate::{config::Config, entity};

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

#[async_trait::async_trait]
impl Presenter for Vec<Notification> {
    async fn set_response(
        self,
        request: &mut Request<Body>,
        resp: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = serde_json::to_string(&self).expect("serialize json");

        resp.set_status(StatusCode::OK).unwrap();
        resp.set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        resp.set_body(serialized.into());

        Ok(())
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
