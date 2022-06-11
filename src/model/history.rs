use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use hyper::{header, Body, Request, Response, StatusCode};
use itertools::Itertools;
use madome_sdk::api::{header::take_origin_response, library, Token};
use serde::Serialize;
use util::http::SetResponse;
use uuid::Uuid;

use crate::{config::Config, entity};

use super::Presenter;

pub enum History {
    Book {
        book_id: u32,
        page: usize,
        user_id: Uuid,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
}

impl History {
    pub fn book_id(&self) -> Option<u32> {
        match self {
            Self::Book { book_id, .. } => Some(*book_id),
            // _ => None,
        }
    }
}

impl From<entity::History> for History {
    fn from(history: entity::History) -> Self {
        match history {
            entity::History::Book {
                book_id,
                page,
                user_id,
                created_at,
                updated_at,
            } => Self::Book {
                book_id,
                page,
                user_id,
                created_at,
                updated_at,
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReducedHistory {
    Book {
        book_id: u32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
}

impl From<History> for ReducedHistory {
    fn from(history: History) -> Self {
        match history {
            History::Book {
                book_id,
                created_at,
                updated_at,
                ..
            } => Self::Book {
                book_id,
                created_at,
                updated_at,
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtendedHistory {
    Book {
        book_id: u32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        book: library::model::Book,
    },
}

#[async_trait::async_trait]
impl Presenter for Vec<History> {
    async fn set_response(
        self,
        request: &mut Request<Body>,
        resp: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = match take_origin_response(request.headers()) {
            true => {
                let histories = self.into_iter().map_into().collect::<Vec<ReducedHistory>>();
                serde_json::to_vec(&histories).expect("json serialize")
            }

            false => {
                // TODO: kind가 추가로 생기게 되면 like과 비슷하게 hashmap에서 값을 빼오는 형식으로 순서를 같게 하면 될 듯
                let book_ids = self.iter().filter_map(|x| x.book_id()).collect::<Vec<_>>();

                let mut books = if book_ids.is_empty() {
                    Vec::new()
                } else {
                    library::def::get_books_by_ids(config.library_url(), Token::default(), book_ids)
                        .await?
                }
                .into_iter()
                .map(|x| (x.id, x))
                .collect::<HashMap<_, _>>();

                let histories = self
                    .into_iter()
                    .filter_map(|x| x.book_id().and_then(|k| Some((x, books.remove(&k)?))))
                    .map(|(x, book)| match x {
                        History::Book {
                            book_id,
                            created_at,
                            updated_at,
                            ..
                        } => ExtendedHistory::Book {
                            book_id,
                            created_at,
                            updated_at,
                            book,
                        },
                    })
                    .collect::<Vec<_>>();

                serde_json::to_vec(&histories).expect("json serialize")
            }
        };

        resp.set_status(StatusCode::OK).unwrap();
        resp.set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        resp.set_body(serialized.into());

        Ok(())
    }
}
