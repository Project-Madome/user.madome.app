mod user;

pub use user::User;

use hyper::http::response::Builder as ResponseBuilder;

use crate::{into_model, model};

into_model![(User, model::User),];

pub trait Presenter: Sized {
    fn to_http(self, _response: ResponseBuilder) -> hyper::Response<hyper::Body> {
        unimplemented!()
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
