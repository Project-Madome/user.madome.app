use hyper::Request;
use std::{collections::HashMap, sync::Arc};
use util::{
    validate::{number, ValidatorNumberExt},
    FromRequest,
};
use uuid::Uuid;

use crate::{
    entity::{like::LikeSortBy, LikeKind, Sort},
    error::UseCaseError,
    model,
    repository::{r#trait::LikeRepository, RepositorySet},
};

#[derive(Clone)]
pub struct Payload {
    pub kind: Option<LikeKind>,
    pub user_id: Uuid,
    pub offset: usize,
    pub page: usize,
    pub sort_by: LikeSortBy,
}

impl Payload {
    pub fn validate(self) -> crate::Result<Self> {
        Ok(Self {
            user_id: self.user_id,
            kind: self.kind,
            offset: self
                .offset
                .validate()
                .max(25)
                .take()
                .map_err(Error::InvalidOffset)?,
            page: self.page,
            sort_by: self.sort_by,
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
        let mut qs = querystring::querify(request.uri().query().unwrap_or_default())
            .into_iter()
            .collect::<HashMap<_, _>>();

        let kind = match qs.remove("kind").unwrap_or("") {
            "book" => Some(LikeKind::Book),
            "book-tag" => Some(LikeKind::BookTag),
            _ => None,
        };

        let offset = qs.get("offset").and_then(|x| x.parse().ok()).unwrap_or(25);

        let page = qs.get("page").and_then(|x| x.parse().ok()).unwrap_or(1);

        let sort_by = match qs.remove("sort-by").unwrap_or("created-at-desc") {
            "created-at-desc" => LikeSortBy::CreatedAt(Sort::Desc),
            "created-at-asc" => LikeSortBy::CreatedAt(Sort::Asc),
            "random" => LikeSortBy::Random,
            x => return Err(Error::InvalidSortBy(x.to_string()).into()),
        };

        log::debug!("sort-by = {sort_by:?}");

        Ok(Self {
            kind,
            user_id,
            offset,
            page,
            sort_by,
        })
    }
}

pub type Model = Vec<model::Like>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("offset: {0}")]
    InvalidOffset(number::Error<usize>),
    #[error("sort-by: ")]
    InvalidSortBy(String),
}

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
    } = p.validate()?;

    let r = repository
        .like()
        .get_many(user_id, kind, offset, page, sort_by)
        .await?;

    Ok(r.into_iter().map(Into::into).collect())
}
