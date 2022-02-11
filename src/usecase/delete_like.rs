use hyper::{Body, Request};
use serde::Deserialize;
use std::sync::Arc;
use util::{r#async::AsyncTryFrom, FromRequest};
use uuid::Uuid;

use crate::{
    entity::Like,
    error::UseCaseError,
    msg::Wrap,
    repository::{r#trait::LikeRepository, RepositorySet},
};

#[derive(Clone, Deserialize)]
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

impl Payload {
    pub fn set_user_id(&mut self, user_id: Uuid) {
        match self {
            Self::Book {
                user_id: exists, ..
            } => {
                *exists = user_id;
            }
            Self::BookTag {
                user_id: exists, ..
            } => {
                *exists = user_id;
            }
        }
    }
}

#[async_trait::async_trait]
impl FromRequest for Payload {
    type Error = crate::Error;
    type Parameter = Uuid;

    async fn from_request(user_id: Uuid, request: Request<Body>) -> Result<Self, Self::Error> {
        let mut r: Payload = Wrap::async_try_from(request).await?.inner();
        r.set_user_id(user_id);

        Ok(r)
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found like")]
    NotFoundLike,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(payload: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    let like = match payload {
        Payload::Book { book_id, user_id } => Like::book(user_id, book_id),

        Payload::BookTag {
            tag_kind,
            tag_name,
            user_id,
        } => Like::book_tag(user_id, tag_kind, tag_name),
    };

    let removed = repository.like().remove(like).await?;

    if !removed {
        return Err(Error::NotFoundLike.into());
    }

    Ok(Model)
}
