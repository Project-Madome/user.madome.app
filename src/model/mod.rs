mod user;

pub use user::User;

use hyper::http::response::Builder as ResponseBuilder;

use crate::{into_model, model, usecase::create_user};

into_model![(User, model::User), (CreateUser, create_user::Model),];

pub trait Presenter: Sized {
    fn to_http(self, _response: ResponseBuilder) -> hyper::Response<hyper::Body> {
        unimplemented!()
    }
}

impl Presenter for create_user::Model {
    fn to_http(self, response: ResponseBuilder) -> hyper::Response<hyper::Body> {
        response.status(201).body(hyper::Body::empty()).unwrap()
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
