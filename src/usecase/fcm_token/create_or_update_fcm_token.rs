use hyper::{Body, Request};
use serde::Deserialize;
use std::sync::Arc;
use util::{BodyParser, FromRequest};
use uuid::Uuid;

use crate::{
    entity::fcm_token::FcmToken,
    error::UseCaseError,
    repository::{r#trait::FcmTokenRepository, RepositorySet},
};

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub udid: Uuid,
    pub fcm_token: String,
    #[serde(default)]
    pub user_id: Uuid,
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Payload {
    type Error = crate::Error;
    type Parameter = Uuid;

    async fn from_request(
        user_id: Self::Parameter,
        request: &'a mut Request<Body>,
    ) -> Result<Self, Self::Error> {
        let payload = request.body_parse().await?;

        Ok(Self { user_id, ..payload })
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

pub async fn execute(
    Payload {
        udid,
        fcm_token,
        user_id,
    }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let _r = repository
        .fcm_token()
        .add_or_update(FcmToken::new(udid, user_id, fcm_token))
        .await?;

    Ok(Model)
}
