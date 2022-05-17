use std::sync::Arc;

use hyper::{Body, Request};
use serde::Deserialize;
use util::{BodyParser, FromRequest};
use uuid::Uuid;

use crate::{
    entity::Like,
    error::UseCaseError,
    repository::{r#trait::LikeRepository, RepositorySet},
};

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    Book {
        book_id: u32,
        #[serde(default)]
        user_id: Uuid,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
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

            Payload::BookTag {
                user_id: exists, ..
            } => *exists = user_id,
        }
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Already exists like")]
    AlreadyExistsLike,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(p: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    use Payload::*;

    match p {
        Book { book_id, user_id } => {
            let saved = repository.like().add(Like::book(user_id, book_id)).await?;

            if !saved {
                return Err(Error::AlreadyExistsLike.into());
            }

            Ok(Model)
        }

        BookTag {
            tag_kind,
            tag_name,
            user_id,
        } => {
            let saved = repository
                .like()
                .add(Like::book_tag(user_id, tag_kind, tag_name))
                .await?;

            if !saved {
                return Err(Error::AlreadyExistsLike.into());
            }

            Ok(Model)
        }
    }
}
