use hyper::Request;
use itertools::Itertools;
use serde::Deserialize;
use std::sync::Arc;
use util::FromRequest;
use uuid::Uuid;

use crate::{
    error::UseCaseError,
    model, payload,
    repository::{
        r#trait::{LikeBy, LikeRepository},
        RepositorySet,
    },
};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PayloadKind {
    Book,
    BookTag,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Temp {
    kind: PayloadKind,
    #[serde(default)]
    user_id: Uuid,
    #[serde(default)]
    ids: Option<Vec<u32>>,
    #[serde(default)]
    tags: Option<Vec<(String, String)>>,
}

impl From<Temp> for Payload {
    fn from(
        Temp {
            kind,
            user_id,
            ids,
            tags,
        }: Temp,
    ) -> Self {
        match kind {
            PayloadKind::Book => Self::Book {
                user_id,
                ids: ids.unwrap_or_default(),
            },

            PayloadKind::BookTag => Self::BookTag {
                user_id,
                tags: tags
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(kind, name)| (kind, name.split('-').join(" ")))
                    .collect(),
            },
        }
    }
}

#[derive(Debug)]
pub enum Payload {
    Book {
        user_id: Uuid,
        ids: Vec<u32>,
    },
    BookTag {
        user_id: Uuid,
        tags: Vec<(String, String)>,
    },
}

impl Payload {
    fn set_user_id(&mut self, x: Uuid) {
        let user_id = match self {
            Self::Book { user_id, .. } => user_id,
            Self::BookTag { user_id, .. } => user_id,
        };

        *user_id = x;
    }

    fn user_id(&self) -> Option<Uuid> {
        match self {
            Self::Book { user_id, .. } if !user_id.is_nil() => Some(*user_id),
            Self::BookTag { user_id, .. } if !user_id.is_nil() => Some(*user_id),

            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Payload {
    type Parameter = Uuid;
    type Error = crate::Error;

    async fn from_request(
        user_id: Self::Parameter,
        request: &'a mut Request<hyper::Body>,
    ) -> Result<Self, Self::Error> {
        let qs = request.uri().query().unwrap_or_default();
        let mut payload: Self = serde_qs::from_str::<Temp>(qs)
            .map_err(payload::Error::QuerystringDeserialize)?
            .into();

        payload.set_user_id(user_id);

        Ok(payload)
    }
}

pub type Model = Vec<model::Like>;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(payload: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    let user_id = payload.user_id();

    match payload {
        Payload::Book { ids, .. } => {
            let r = repository
                .like()
                .get_many_by(user_id, LikeBy::Book { ids })
                .await?;

            Ok(r.into_iter().map_into().collect())
        }

        Payload::BookTag { tags, .. } => {
            let r = repository
                .like()
                .get_many_by(user_id, LikeBy::BookTag { tags })
                .await?;

            Ok(r.into_iter().map_into().collect())
        }
    }
}
