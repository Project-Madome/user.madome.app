mod like;
mod user;

pub use like::Like;
pub use user::User;

use hyper::{http::response::Builder as ResponseBuilder, Body, Response, StatusCode};

use crate::{
    into_model, model,
    usecase::{create_like, create_user, delete_like},
};

into_model![
    (User, model::User),
    (CreateUser, create_user::Model),
    (Likes, Vec<model::Like>),
    (CreateLike, create_like::Model),
    (DeleteLike, delete_like::Model),
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
