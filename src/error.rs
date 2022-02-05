use hyper::{Body, Response, StatusCode};

use crate::usecase::{create_user, get_user};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Msg: {0}")]
    Msg(#[from] MsgError),
    #[error("Command: {0}")]
    Command(#[from] CommandError),
    #[error("UseCase: {0}")]
    UseCase(#[from] UseCaseError),
    #[error("Repository: {0}")]
    Repository(#[from] RepositoryError),

    #[error("AuthSdk: {0}")]
    AuthSdk(#[from] madome_sdk::auth::Error),

    // TODO: 나중에 위치 재선정
    #[error("ReadChunksFromBody: {0}")]
    ReadChunksFromBody(#[from] hyper::Error),
}

type MsgError = crate::msg::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("SeaOrm: {0}")]
    SeaOrm(#[from] sea_orm::DbErr),
}

impl From<sea_orm::DbErr> for crate::Error {
    fn from(error: sea_orm::DbErr) -> Self {
        Error::Repository(error.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {}

#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("GetUser: {0}")]
    GetUser(#[from] get_user::Error),
    #[error("CreateUser: {0}")]
    CreateUser(#[from] create_user::Error),
}

impl From<Error> for Response<Body> {
    fn from(error: Error) -> Self {
        use crate::msg::Error::*;
        use create_user::Error::*;
        use get_user::Error::*;
        use Error::*;
        use UseCaseError::*;

        let response = Response::builder();

        match error {
            Msg(JsonDeserializePayload(err)) => response
                .status(StatusCode::BAD_REQUEST)
                .body(err.to_string().into()),

            Msg(NotFound) => response
                .status(StatusCode::NOT_FOUND)
                .body("Not found".into()),

            UseCase(CreateUser(AlreadyExistUser)) => response
                .status(StatusCode::CONFLICT)
                .body("Already exist user".into()),

            UseCase(GetUser(NotFoundUser)) => response
                .status(StatusCode::NOT_FOUND)
                .body("Not found user".into()),

            AuthSdk(err) => err.to_http(response),

            err => response
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(err.to_string().into()),
        }
        .unwrap()
    }
}
