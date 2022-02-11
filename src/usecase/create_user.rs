use std::sync::Arc;

use serde::Deserialize;
use util::validate::{number, string, ValidatorNumberExt, ValidatorStringExt};

use crate::{
    entity::user::User,
    error::UseCaseError,
    repository::{r#trait::UserRepository, RepositorySet},
};

#[derive(Deserialize, Clone)]
pub struct Payload {
    pub name: String,
    pub email: String,
    #[serde(default)] // default = 0
    pub role: u8,
}

impl Payload {
    pub fn validate(self) -> Result<Self, Error> {
        Ok(Self {
            name: self
                .name
                .validate()
                .min(1)
                .max(20)
                .take()
                .map_err(Error::InvalidName)?,
            email: self
                .email
                .validate()
                .email()
                .take()
                .map_err(Error::InvalidEmail)?,
            role: self
                .role
                .validate()
                .min(0)
                .max(1)
                .take()
                .map_err(Error::InvalidRole)?,
        })
    }
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("name: {0}")]
    InvalidName(string::Error),
    #[error("email: {0}")]
    InvalidEmail(string::Error),
    #[error("role: {0}")]
    InvalidRole(number::Error<u8>),

    #[error("Already exist error")]
    AlreadyExistsUser,
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(p: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    let Payload { name, email, role } = p.validate()?;

    let new_user = User::new(name, email, role.into());

    let maybe_saved = repository.user().add(new_user).await?;

    match maybe_saved {
        Some(_user) => Ok(Model),
        None => Err(Error::AlreadyExistsUser.into()),
    }
}
