use std::sync::Arc;

use hyper::{Body, Request};
use serde::Deserialize;
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

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Payload {
    type Error = crate::Error;
    type Parameter = Uuid;

    async fn from_request(
        user_id: Self::Parameter,
        request: &'a mut Request<Body>,
    ) -> Result<Self, Self::Error> {
        let mut payload: Payload = request.body_parse().await?;

        payload.set_user_id(user_id);

        Ok(payload)
    }
}

impl Payload {
    fn set_user_id(&mut self, user_id: Uuid) {
        match self {
            Payload::Book {
                user_id: exists, ..
            } => *exists = user_id,
        }
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(p: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    use Payload::*;

    match p {
        Book { book_id, user_id } => {
            let _r = repository
                .history()
                .add_or_update(History::book(book_id, user_id))
                .await?;

            Ok(Model)
        }
    }
}
