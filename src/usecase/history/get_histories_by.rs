use hyper::Request;
use serde::Deserialize;
use std::sync::Arc;
use util::FromRequest;
use uuid::Uuid;

use crate::{
    error::UseCaseError,
    model, payload,
    repository::{
        r#trait::{HistoryBy, HistoryRepository},
        RepositorySet,
    },
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum Payload {
    Book {
        #[serde(default)]
        user_id: Uuid,
        ids: Vec<u32>,
    },
}

impl Payload {
    fn set_user_id(&mut self, x: Uuid) {
        match self {
            Self::Book { user_id, .. } => {
                *user_id = x;
            }
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
        let mut payload: Self =
            serde_qs::from_str(qs).map_err(payload::Error::QuerystringDeserialize)?;

        payload.set_user_id(user_id);

        Ok(payload)
    }
}

pub type Model = Vec<model::History>;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(payload: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    match payload {
        Payload::Book { user_id, ids } => {
            let by = HistoryBy::Book { ids };
            let r = repository.history().get_many_by(user_id, by).await?;

            Ok(r.into_iter().map(Into::into).collect())
        }
    }
}
