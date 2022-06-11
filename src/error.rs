use std::sync::Arc;

use hyper::{Body, Request, Response, StatusCode};
use util::{body_parser, http::SetResponse};

use crate::{
    command::{has_book, has_book_tag},
    config::Config,
    model::Presenter,
    payload,
    usecase::{
        create_like, create_notifications, create_or_update_fcm_token, create_or_update_history,
        create_user, delete_history, delete_like, get_fcm_tokens, get_histories, get_histories_by,
        get_likes, get_likes_by, get_notifications, get_user,
    },
};

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

    #[error("Payload: {0}")]
    Payload(#[from] payload::Error),

    #[error("AuthSdk: {0}")]
    AuthSdk(#[from] madome_sdk::api::auth::Error),

    #[error("LibrarySdk: {0}")]
    LibrarySdk(#[from] madome_sdk::api::library::Error),

    // TODO: 나중에 위치 재선정
    #[error("ReadChunksFromBody: {0}")]
    ReadChunksFromBody(#[from] hyper::Error),
}

impl From<body_parser::Error> for Error {
    fn from(err: body_parser::Error) -> Self {
        match err {
            body_parser::Error::JsonDeserialize(e) => payload::Error::JsonDeserialize(e).into(),
            body_parser::Error::NotSupportedContentType(e) => {
                payload::Error::NotSupportedContentType(e).into()
            }
        }
    }
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

impl From<sea_orm::TransactionError<sea_orm::DbErr>> for crate::Error {
    fn from(err: sea_orm::TransactionError<sea_orm::DbErr>) -> Self {
        match err {
            sea_orm::TransactionError::Connection(err) => Self::Repository(err.into()),
            sea_orm::TransactionError::Transaction(err) => Self::Repository(err.into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("HasBook: {0}")]
    HasBook(#[from] has_book::Error),

    #[error("HasBookTag: {0}")]
    HasBookTag(#[from] has_book_tag::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("GetUser: {0}")]
    GetUser(#[from] get_user::Error),
    #[error("CreateUser: {0}")]
    CreateUser(#[from] create_user::Error),

    #[error("GetLikes: {0}")]
    GetLikes(#[from] get_likes::Error),
    #[error("GetLikesBy: {0}")]
    GetLikesBy(#[from] get_likes_by::Error),
    #[error("CreateLike: {0}")]
    CreateLike(#[from] create_like::Error),
    #[error("DeleteLike: {0}")]
    DeleteLike(#[from] delete_like::Error),

    // #[error("GetLikesFromBookTags: {0}")]
    // GetLikesFromBookTags(#[from] get_likes_from_book_tags::Error),
    #[error("CreateNotifications: {0}")]
    CreateNotifications(#[from] create_notifications::Error),
    #[error("GetNotifications: {0}")]
    GetNotifications(#[from] get_notifications::Error),

    #[error("CreateOrUpdateFcmToken: {0}")]
    CreateOrUpdateFcmToken(#[from] create_or_update_fcm_token::Error),
    #[error("GetFcmTokens: {0}")]
    GetFcmTokens(#[from] get_fcm_tokens::Error),

    #[error("CreateOrUpdateHistory: {0}")]
    CreateOrUpdateHistory(#[from] create_or_update_history::Error),
    #[error("GetHistories: {0}")]
    GetHistories(#[from] get_histories::Error),
    #[error("GetHistoriesBy: {0}")]
    GetHistoriesBy(#[from] get_histories_by::Error),
    #[error("DeleteHistory: {0}")]
    DeleteHistory(#[from] delete_history::Error),
}

#[async_trait::async_trait]
impl Presenter for Error {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        resp: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        use crate::msg::Error::*;
        use create_like::Error::*;
        use create_user::Error::*;
        use delete_history::Error::*;
        use delete_like::Error::*;
        use get_user::Error::*;
        use Error::*;
        use UseCaseError::*;

        match self {
            Msg(NotFound) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body("Not found".into());
            }

            Payload(err) => {
                resp.set_status(StatusCode::BAD_REQUEST).unwrap();
                resp.set_body(err.to_string().into());
            }

            UseCase(CreateUser(
                err @ InvalidName(_) | err @ InvalidEmail(_) | err @ InvalidRole(_),
            )) => {
                resp.set_status(StatusCode::BAD_REQUEST).unwrap();
                resp.set_body(err.to_string().into());
            }

            UseCase(CreateUser(AlreadyExistsUser)) => {
                resp.set_status(StatusCode::CONFLICT).unwrap();
                resp.set_body("Already exist user".into());
            }
            UseCase(GetUser(NotFoundUser)) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body("Not found user".into());
            }

            UseCase(CreateLike(err @ AlreadyExistsLike)) => {
                resp.set_status(StatusCode::CONFLICT).unwrap();
                resp.set_body(err.to_string().into());
            }
            UseCase(CreateLike(
                err @ create_like::Error::NotFoundBook | err @ create_like::Error::NotFoundBookTag,
            )) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body(err.to_string().into());
            }
            UseCase(DeleteLike(err @ NotFoundLike)) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body(err.to_string().into());
            }

            UseCase(CreateOrUpdateHistory(err @ create_or_update_history::Error::NotFoundBook)) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body(err.to_string().into());
            }
            UseCase(DeleteHistory(err @ NotFoundHistory)) => {
                resp.set_status(StatusCode::NOT_FOUND).unwrap();
                resp.set_body(err.to_string().into())
            }

            AuthSdk(ref err) => {
                use madome_sdk::api::{auth::Error as AuthError, BaseError};

                match err {
                    AuthError::Base(err) => match err {
                        err @ BaseError::Unauthorized => {
                            resp.set_status(StatusCode::UNAUTHORIZED).unwrap();
                            resp.set_body(err.to_string().into());
                        }
                        err @ BaseError::PermissionDenied => {
                            resp.set_status(StatusCode::FORBIDDEN).unwrap();
                            resp.set_body(err.to_string().into());
                        }
                        BaseError::Undefined(code, body) => {
                            resp.set_status(code).unwrap();
                            resp.set_body(body.to_owned().into());
                        }
                        _ => {
                            resp.set_status(StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                            resp.set_body(err.to_string().into());
                        }
                    },
                    _ => {
                        resp.set_status(StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                        resp.set_body(err.to_string().into());
                    }
                }
            }

            err => {
                resp.set_status(StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                resp.set_body(err.to_string().into());
            }
        };

        Ok(())
    }
}
