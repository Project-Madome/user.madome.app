use std::sync::Arc;

use hyper::{Body, Request};
use serde::Deserialize;
use util::{validate::ValidatorNumberExt, BodyParser, FromRequest};
use uuid::Uuid;

use crate::{
    command::CommandSet,
    entity::History,
    error::UseCaseError,
    payload,
    repository::{r#trait::HistoryRepository, RepositorySet},
};

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    Book {
        book_id: u32,
        page: usize,
        #[serde(default)]
        user_id: Uuid,
    },
}

impl Payload {
    fn check(self) -> crate::Result<Self> {
        match self {
            Self::Book {
                book_id,
                page,
                user_id,
            } => {
                let book_id = book_id
                    .validate()
                    .min(1)
                    .take()
                    .map_err(payload::Error::InvalidBookId)?;

                let page = page
                    .validate()
                    .min(1)
                    .take()
                    .map_err(payload::Error::InvalidPage)?;

                Ok(Self::Book {
                    book_id,
                    page,
                    user_id,
                })
            }
        }
    }
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

        payload.check()
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
pub enum Error {
    #[error("Not found book in library")]
    NotFoundBook,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    p: Payload,
    repository: Arc<RepositorySet>,
    command: Arc<CommandSet>,
) -> crate::Result<Model> {
    use Payload::*;

    match p {
        Book {
            book_id,
            page,
            user_id,
        } => {
            let has_book = command.has_book(book_id).await?;

            if !has_book {
                return Err(Error::NotFoundBook.into());
            }

            let _r = repository
                .history()
                .add_or_update(History::book(book_id, page, user_id))
                .await?;

            Ok(Model)
        }
    }
}
