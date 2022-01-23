use std::sync::Arc;

use crate::{
    error::UseCaseError,
    model,
    repository::{r#trait::UserRepository, RepositorySet},
    utils::http::url::PathVariable,
};

#[derive(Clone)]
pub struct Payload {
    pub id_or_email: String,
}

impl From<PathVariable> for Payload {
    fn from(mut path_variable: PathVariable) -> Self {
        let id_or_email = path_variable.next_variable().unwrap_or_default();

        Self { id_or_email }
    }
}

pub type Model = model::User;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    NotFoundUser,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload { id_or_email }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let maybe_user = repository.user().get(id_or_email).await?;

    match maybe_user {
        Some(user) => Ok(user.into()),
        None => Err(Error::NotFoundUser.into()),
    }
}
