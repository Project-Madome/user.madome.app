use hyper::{Body, Response, StatusCode};

use crate::usecase::{add_user, get_user};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Msg Layer: {0}")]
    Msg(#[from] MsgError),
    #[error("Command Layer: {0}")]
    Command(#[from] CommandError),
    #[error("UseCase Layer: {0}")]
    UseCase(#[from] UseCaseError),
    #[error("Repository Layer: {0}")]
    Repository(#[from] RepositoryError),

    // TODO: 나중에 위치 재선정
    #[error("ReadChunksFromBody")]
    ReadChunksFromBody(#[from] hyper::Error),
}

type MsgError = crate::msg::Error;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {}

#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("")]
    GetUser(#[from] get_user::Error),
    #[error("")]
    AddUser(#[from] add_user::Error),
}

impl From<Error> for Response<Body> {
    fn from(error: Error) -> Self {
        use crate::msg::Error::*;
        use add_user::Error::*;
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

            UseCase(AddUser(AlreadyExistUser)) => response
                .status(StatusCode::CONFLICT)
                .body("Already exist user".into()),

            UseCase(GetUser(NotFoundUser)) => response
                .status(StatusCode::NOT_FOUND)
                .body("Not found user".into()),

            _ => response
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty()),
        }
        .unwrap()
    }
}
