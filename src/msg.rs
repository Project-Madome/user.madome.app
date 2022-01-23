use hyper::{Body, Method, Request};
use serde::de::DeserializeOwned;

use crate::{
    usecase::{add_user, get_user},
    utils::{
        http::url::{is_path_variable, PathVariable},
        r#async::{AsyncTryFrom, AsyncTryInto},
        ReadChunks,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    NotFound,
    #[error("")]
    JsonDeserializePayload(serde_json::Error),
}

/// Msg의 Payload는 같은 이름의 usecase의 Payload와는 관계가 없음
///
/// Msg의 Payload는 실행되어야하는 usecase 순서에 따라 정해짐 (제일 처음 실행하는 usecase의 Payload)
///
/// 실행되는 순서는 Resolver 참조
pub enum Msg {
    GetUser(get_user::Payload),
    AddUser(add_user::Payload),
}

fn matcher(req_path: &str, pattern: &str) -> bool {
    let mut origin = req_path.split('/');
    let pats = pattern.split('/');

    for pat in pats {
        if let Some(origin) = origin.next() {
            if !is_path_variable(pat) && pat != origin {
                return false;
            }
        } else {
            return false;
        }
    }

    origin.next().is_none()
}

#[async_trait::async_trait]
impl AsyncTryFrom<Request<Body>> for Msg {
    type Error = crate::Error;

    async fn async_try_from(request: Request<Body>) -> Result<Self, Self::Error> {
        let method = request.method().clone();
        let path = request.uri().path();

        let msg = match (method, path) {
            (Method::POST, "/users") => Msg::AddUser(request.async_try_into().await?),
            (Method::GET, path) if matcher(path, "/users/:user_id") => {
                Msg::GetUser(PathVariable::from((path, "/users/:user_id")).into())
            }
            _ => return Err(Error::NotFound.into()),
        };

        Ok(msg)
    }
}

#[async_trait::async_trait]
impl<P> AsyncTryFrom<Request<Body>> for P
where
    P: DeserializeOwned,
{
    type Error = crate::Error;

    async fn async_try_from(mut request: Request<Body>) -> Result<Self, Self::Error> {
        let chunks = request.body_mut().read_chunks().await?;

        let payload =
            serde_json::from_slice::<P>(&chunks).map_err(Error::JsonDeserializePayload)?;

        Ok(payload)
    }
}
