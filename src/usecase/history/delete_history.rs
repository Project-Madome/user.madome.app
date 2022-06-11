use hyper::{Body, Request};
use serde::Deserialize;
use std::sync::Arc;
use util::{BodyParser, FromRequest};
use uuid::Uuid;

use crate::{
    entity::History,
    error::UseCaseError,
    repository::{r#trait::HistoryRepository, RepositorySet},
};

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    Book {
        book_id: u32,
        #[serde(default)]
        user_id: Uuid,
    },
}

impl Payload {
    fn set_user_id(&mut self, user_id: Uuid) {
        match self {
            Self::Book {
                user_id: exists, ..
            } => {
                *exists = user_id;
            }
        }
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Payload {
    type Error = crate::Error;
    type Parameter = Uuid;

    async fn from_request(
        user_id: Uuid,
        request: &'a mut Request<Body>,
    ) -> Result<Self, Self::Error> {
        let mut r: Payload = request.body_parse().await?;
        r.set_user_id(user_id);

        Ok(r)
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found history")]
    NotFoundHistory,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(payload: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    let history = match payload {
        Payload::Book { book_id, user_id } => History::book(book_id, 1, user_id),
    };

    let removed = repository.history().remove(history).await?;

    if !removed {
        return Err(Error::NotFoundHistory.into());
    }

    Ok(Model)
}
