use hyper::{Body, Request};
use itertools::Itertools;
use std::sync::Arc;

use crate::{
    error::UseCaseError,
    model,
    repository::{r#trait::LikeRepository, RepositorySet},
};

#[derive(Debug)]
pub struct Payload {
    pub book_tags: Vec<(String, String)>,
}

impl TryFrom<Request<Body>> for Payload {
    type Error = crate::Error;

    fn try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let qs = querystring::querify(request.uri().query().unwrap_or_default());

        let tags = qs
            .iter()
            .filter(|(k, _)| *k == "tags[]")
            .map(|(_, v)| v)
            .map(|v| v.split('-'))
            .filter_map(|mut v| Some((v.next()?.to_owned(), v.join(" "))))
            .collect::<Vec<_>>();

        // log::debug!("{qs:?}");
        log::debug!("{tags:?}");

        Ok(Self { book_tags: tags })
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

pub async fn execute(
    Payload { book_tags }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let r = repository.like().get_many_by_book_tags(book_tags).await?;

    Ok(r.into_iter().map(Into::into).collect())
}
