use std::sync::Arc;

use serde::Deserialize;

use crate::{
    entity::user::{User, UserRole},
    error::UseCaseError,
    model,
    repository::{r#trait::UserRepository, RepositorySet},
};

#[derive(Deserialize, Clone)]
pub struct Payload {
    pub name: String,
    pub email: String,
    // role: u8,
}

pub type Model = model::User;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Already exist error")]
    AlreadyExistUser,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(
    Payload { name, email }: Payload,
    repository: Arc<RepositorySet>,
) -> crate::Result<Model> {
    let new_user = User::new(name, email, UserRole::Normal);

    let maybe_saved = repository.user().add(new_user).await?;

    match maybe_saved {
        Some(user) => Ok(user.into()),
        None => Err(Error::AlreadyExistUser.into()),
    }
}
