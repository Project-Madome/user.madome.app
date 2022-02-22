use hyper::{Body, Request};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::UseCaseError,
    payload,
    repository::{r#trait::FcmTokenRepository, RepositorySet},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Payload {
    pub user_ids: Vec<Uuid>,
}

impl TryFrom<&mut Request<Body>> for Payload {
    type Error = crate::Error;

    fn try_from(request: &mut Request<Body>) -> Result<Self, Self::Error> {
        let a = request.uri().query().unwrap_or_default();

        let payload = serde_qs::from_str(a).map_err(payload::Error::QuerystringDeserialize)?;

        Ok(payload)
    }
}

pub struct Model(pub Vec<String>);

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload { user_ids }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let fcm_tokens = repository.fcm_token().get_many(user_ids).await?;

    Ok(Model(fcm_tokens))
}
