use std::sync::Arc;

use hyper::{Body, Request};
use serde::Deserialize;
use util::{validate::ValidatorNumberExt, FromRequest};
use uuid::Uuid;

use crate::{
    error::UseCaseError,
    model,
    payload::{
        self,
        notification::{NotificationKind, NotificationSortBy},
    },
    repository::{r#trait::NotificationRepository, RepositorySet},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Payload {
    #[serde(default)]
    pub user_id: Uuid,
    pub kind: Option<NotificationKind>,
    pub per_page: Option<usize>,
    pub page: Option<usize>,
    pub sort_by: Option<NotificationSortBy>,
}

impl Payload {
    pub fn check(self) -> crate::Result<Self> {
        let per_page = self
            .per_page
            .unwrap_or(25)
            .validate()
            .min(1)
            .max(100)
            .take()
            .map_err(payload::Error::InvalidPerPage)?;

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
            per_page: Some(per_page),
            page: Some(page),
            sort_by: Some(self.sort_by.unwrap_or(NotificationSortBy::CreatedAtDesc)),
        })
    }
}

#[async_trait::async_trait]
impl<'a> FromRequest<'a> for Payload {
    type Parameter = Uuid;
    type Error = crate::Error;

    async fn from_request(
        user_id: Self::Parameter,
        request: &'a mut Request<Body>,
    ) -> Result<Self, Self::Error> {
        let qs = request.uri().query().unwrap_or_default();
        let payload: Payload =
            serde_qs::from_str(qs).map_err(payload::Error::QuerystringDeserialize)?;

        Ok(Self {
            user_id,
            ..payload.check()?
        })
    }
}

pub type Model = Vec<model::Notification>;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload {
        user_id,
        kind,
        per_page,
        page,
        sort_by,
    }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let notifications = repository
        .notification()
        .get_many(
            user_id,
            kind.map(Into::into),
            per_page.unwrap(),
            page.unwrap(),
            sort_by.unwrap().into(),
        )
        .await?;

    Ok(notifications.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod payload_tests {
    use hyper::{Body, Request};
    use util::ToPayload;
    use uuid::Uuid;

    use crate::payload::notification::{NotificationKind, NotificationSortBy};

    use super::Payload;

    pub const USER_ID: Uuid = Uuid::nil();

    fn request(uri: &str) -> Request<Body> {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn default() {
        let mut request = request("/");

        let payload: Payload = request.to_payload(USER_ID).await.unwrap();

        let expected = Payload {
            per_page: Some(25),
            page: Some(1),
            sort_by: Some(NotificationSortBy::CreatedAtDesc),
            kind: None,
            user_id: USER_ID,
        };

        assert_eq!(payload, expected);
    }

    #[tokio::test]
    async fn inject() {
        let mut request = request("/?per-page=11&page=3&sort-by=created-at-asc&kind=book");

        let payload: Payload = request.to_payload(USER_ID).await.unwrap();

        let expected = Payload {
            per_page: Some(11),
            page: Some(3),
            sort_by: Some(NotificationSortBy::CreatedAtAsc),
            kind: Some(NotificationKind::Book),
            user_id: USER_ID,
        };

        assert_eq!(payload, expected);
    }
}
