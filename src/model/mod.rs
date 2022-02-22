mod like;
mod notification;
mod user;

use std::sync::Arc;

pub use like::{Like, LikeWithoutUserId};
pub use notification::Notification;
pub use user::User;

use hyper::{header, Body, Request, Response, StatusCode};
use util::http::SetResponse;

use crate::{
    config::Config,
    into_model, model,
    usecase::{
        create_like, create_notifications, create_or_update_fcm_token, create_user, delete_like,
        get_fcm_tokens,
    },
};

into_model![
    (User, model::User),
    (CreateUser, create_user::Model),
    //
    (Likes, Vec<model::Like>),
    (CreateLike, create_like::Model),
    (DeleteLike, delete_like::Model),
    //
    (Notifications, Vec<model::Notification>),
    (CreateNotifications, create_notifications::Model),
    //
    (CreateOrUpdateFcmToken, create_or_update_fcm_token::Model),
    (GetFcmTokens, get_fcm_tokens::Model),
];

#[async_trait::async_trait]
pub trait Presenter: Sized {
    /// &mut Request를 받는 이유는 핸들러에서 body parse하는 과정에서 mutable이 필요하기 때문임
    async fn set_response(
        self,
        request: &mut Request<Body>,
        response: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<()>;
}

#[async_trait::async_trait]
impl Presenter for create_user::Model {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        response: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        response.set_status(StatusCode::CREATED).unwrap();
        response.set_body(Body::empty());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Presenter for create_like::Model {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        response: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        response.set_status(StatusCode::CREATED).unwrap();
        response.set_body(Body::empty());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Presenter for delete_like::Model {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        response: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        response.set_status(StatusCode::NO_CONTENT).unwrap();
        response.set_body(Body::empty());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Presenter for create_notifications::Model {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        response: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        response.set_status(StatusCode::CREATED).unwrap();
        response.set_body(Body::empty());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Presenter for create_or_update_fcm_token::Model {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        response: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        response.set_status(StatusCode::CREATED).unwrap();
        response.set_body(Body::empty());

        Ok(())
    }
}

#[async_trait::async_trait]
impl Presenter for get_fcm_tokens::Model {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        response: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = serde_json::to_string(&self.0).expect("json serialize");

        response.set_status(StatusCode::OK).unwrap();
        response
            .set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        response.set_body(serialized.into());

        Ok(())
    }
}

#[macro_export]
macro_rules! into_model {
    ($(($member:ident, $from:ty)),*,) => {
        pub enum Model {
            $(
                $member($from),
            )*
        }

        $(
            impl From<$from> for Model {
                fn from(from: $from) -> Model {
                    Model::$member(from)
                }
            }
        )*


        #[async_trait::async_trait]
        impl Presenter for Model {
            async fn set_response(self, request: &mut Request<Body>, response: &mut Response<Body>, config: Arc<Config>) -> crate::Result<()> {
                use Model::*;

                match self {
                    $(
                        $member(model) => model.set_response(request, response, config).await,
                    )*
                }
            }
        }

    };
}
