mod like;
mod notification;
mod user;

pub use like::{Like, LikeWithoutUserId};
pub use notification::Notification;
pub use user::User;

use hyper::{header, http::response::Builder as ResponseBuilder, Body, Response, StatusCode};

use crate::{
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

pub trait Presenter: Sized {
    fn to_http(self, _response: ResponseBuilder) -> Response<Body> {
        unimplemented!()
    }
}

impl Presenter for create_user::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        response.status(201).body(Body::empty()).unwrap()
    }
}

impl Presenter for create_like::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        response.status(201).body(Body::empty()).unwrap()
    }
}

impl Presenter for delete_like::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        response
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap()
    }
}

impl Presenter for create_notifications::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        response
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap()
    }
}

impl Presenter for create_or_update_fcm_token::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        response
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap()
    }
}

impl Presenter for get_fcm_tokens::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        let serialized = serde_json::to_string(&self.0).expect("json serialize");

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
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


        impl Presenter for Model {
            fn to_http(self, response: ResponseBuilder) -> hyper::Response<hyper::Body> {
                use Model::*;

                match self {
                    $(
                        $member(model) => model.to_http(response),
                    )*
                }
            }
        }

    };
}
