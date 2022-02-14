use hyper::Request;
use serde::Deserialize;
use std::sync::Arc;
use util::{validate::ValidatorNumberExt, FromRequest};
use uuid::Uuid;

use crate::{
    error::UseCaseError,
    model,
    payload::{
        self,
        like::{LikeKind, LikeSortBy},
    },
    repository::{r#trait::LikeRepository, RepositorySet},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Payload {
    #[serde(default)]
    pub user_id: Uuid,
    pub kind: Option<LikeKind>,
    pub offset: Option<usize>,
    pub page: Option<usize>,
    pub sort_by: Option<LikeSortBy>,
}

impl Payload {
    pub fn check(self) -> crate::Result<Self> {
        let offset = self
            .offset
            .unwrap_or(25)
            .validate()
            .min(1)
            .max(100)
            .take()
            .map_err(payload::Error::InvalidOffset)?;

        let page = self
            .page
            .unwrap_or(1)
            .validate()
            .min(1)
            .take()
            .map_err(payload::Error::InvalidPage)?;

        Ok(Self {
            user_id: self.user_id,
            kind: self.kind,
            offset: Some(offset),
            page: Some(page),
            sort_by: Some(self.sort_by.unwrap_or(LikeSortBy::CreatedAtDesc)),
        })
    }
}

#[async_trait::async_trait]
impl FromRequest for Payload {
    type Parameter = Uuid;
    type Error = crate::Error;

    async fn from_request(
        user_id: Self::Parameter,
        request: Request<hyper::Body>,
    ) -> Result<Self, Self::Error> {
        let qs = request.uri().query().unwrap_or_default();
        let payload: Self =
            serde_qs::from_str(qs).map_err(payload::Error::QuerystringDeserialize)?;

        Ok(Self {
            user_id,
            ..payload.check()?
        })
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

pub async fn execute(p: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    let Payload {
        kind,
        user_id,
        offset,
        page,
        sort_by,
    } = p.check()?;

    let r = repository
        .like()
        .get_many(
            user_id,
            kind.map(Into::into),
            offset.unwrap(),
            page.unwrap(),
            sort_by.unwrap().into(),
        )
        .await?;

    Ok(r.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod payload_tests {
    use hyper::{Body, Request};
    use util::IntoPayload;
    use uuid::Uuid;

    use crate::payload::like::{LikeKind, LikeSortBy};

    use super::Payload;

    pub const USER_ID: Uuid = Uuid::nil();

    fn request(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn default() {
        let request = request("/");

        let payload: Payload = request.into_payload(USER_ID).await.unwrap();

        let expected = Payload {
            offset: Some(25),
            page: Some(1),
            sort_by: Some(LikeSortBy::CreatedAtDesc),
            kind: None,
            user_id: USER_ID,
        };

        assert_eq!(payload, expected);
    }

    #[tokio::test]
    async fn inject() {
        let request = request("/?offset=17&page=11&sort-by=created-at-asc&kind=book");

        let payload: Payload = request.into_payload(USER_ID).await.unwrap();

        let expected = Payload {
            offset: Some(17),
            page: Some(11),
            sort_by: Some(LikeSortBy::CreatedAtAsc),
            kind: Some(LikeKind::Book),
            user_id: USER_ID,
        };

        assert_eq!(payload, expected);
    }
}
