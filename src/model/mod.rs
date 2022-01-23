mod user;

pub use user::User;

use crate::{into_model, model};

into_model![(User, model::User),];

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


        impl From<Model> for hyper::Response<hyper::Body> {
            fn from(model: Model) -> Self {
                use Model::*;

                match model {
                    $(
                        $member(model) => model.into(),
                    )*
                }
            }
        }

    };
}
