use std::sync::Arc;

use hyper::{http::response::Builder as ResponseBuilder, Body, Method, Request};
use madome_sdk::auth::{Auth, Role, MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN};
use serde::de::DeserializeOwned;
use util::{
    http::{
        url::{is_path_variable, PathVariable},
        Cookie, SetHeaders,
    },
    r#async::AsyncTryFrom,
    IntoPayload, ReadChunks,
};

use crate::{
    config::Config,
    usecase::{
        create_like, create_user, delete_like, get_likes, get_likes_from_book_tags, get_user,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Json deserialize")]
    JsonDeserializePayload(serde_json::Error),
}

/// Msg의 Payload는 같은 이름의 usecase의 Payload와는 관계가 없음
///
/// Msg의 Payload는 실행되어야하는 usecase 순서에 따라 정해짐 (제일 처음 실행하는 usecase의 Payload)
///
/// 실행되는 순서는 Resolver 참조
pub enum Msg {
    CreateUser(create_user::Payload),
    GetUser(get_user::Payload),
    CreateLike(create_like::Payload),
    GetLikes(get_likes::Payload),
    DeleteLike(delete_like::Payload),
    GetLikesFromBookTags(get_likes_from_book_tags::Payload),
}

impl Msg {
    pub async fn http(
        request: Request<Body>,
        mut response: ResponseBuilder,
        config: Arc<Config>,
    ) -> crate::Result<(Self, ResponseBuilder)> {
        use Role::*;

        let method = request.method().clone();
        let path = request.uri().path();
        let cookie = Cookie::from(request.headers());

        let access_token = cookie.get(MADOME_ACCESS_TOKEN).unwrap_or_default();
        let refresh_token = cookie.get(MADOME_REFRESH_TOKEN).unwrap_or_default();

        let auth = Auth::new(config.madome_auth_url());

        let (r, msg) = match (method, path) {
            /* Public */
            (Method::POST, "/users") => {
                let (_, maybe_token_pair) = auth
                    .check_and_refresh_token_pair(access_token, refresh_token, Developer)
                    .await?;

                let p = Wrap::async_try_from(request).await?.inner();

                (Some(maybe_token_pair), Msg::CreateUser(p))
            }

            (Method::GET, "/users/@me") => {
                let (r, maybe_token_pair) = auth
                    .check_and_refresh_token_pair(access_token, refresh_token, Normal)
                    .await?;

                let p = get_user::Payload {
                    id_or_email: r.user_id.to_string(),
                };

                (Some(maybe_token_pair), Msg::GetUser(p))
            }

            (Method::POST, "/users/@me/likes") => {
                let (r, maybe_token_pair) = auth
                    .check_and_refresh_token_pair(access_token, refresh_token, Normal)
                    .await?;

                let mut p: create_like::Payload = Wrap::async_try_from(request).await?.inner();
                p.set_user_id(r.user_id);

                (Some(maybe_token_pair), Msg::CreateLike(p))
            }

            (Method::GET, "/users/@me/likes") => {
                let (r, maybe_token_pair) = auth
                    .check_and_refresh_token_pair(access_token, refresh_token, Normal)
                    .await?;

                let p = request.into_payload(r.user_id).await?;

                (Some(maybe_token_pair), Msg::GetLikes(p))
            }

            (Method::DELETE, "/users/@me/likes") => {
                let (r, maybe_token_pair) = auth
                    .check_and_refresh_token_pair(access_token, refresh_token, Normal)
                    .await?;

                let p = request.into_payload(r.user_id).await?;

                (Some(maybe_token_pair), Msg::DeleteLike(p))
            }

            /* Internal */
            (Method::GET, path) if matcher(path, "/users/:user_id_or_email") => {
                auth.check_internal(request.headers())?;

                let p: get_user::Payload =
                    PathVariable::from((path, "/users/:user_id_or_email")).into();

                (None, Msg::GetUser(p))
            }

            (Method::GET, "/users/likes/book-tags") => {
                auth.check_internal(request.headers())?;

                let p = request.try_into()?;

                (None, Msg::GetLikesFromBookTags(p))
            }

            _ => return Err(Error::NotFound.into()),
        };

        if let Some(set_cookie) = r {
            // response에 쿠키 설정하고 response 넘겨줌
            response = response.headers(set_cookie.iter());
        }

        Ok((msg, response))
    }
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

pub struct Wrap<P>(pub P);

impl<P> Wrap<P> {
    pub fn inner(self) -> P {
        self.0
    }
}

#[async_trait::async_trait]
impl<P> AsyncTryFrom<Request<Body>> for Wrap<P>
where
    P: DeserializeOwned,
{
    type Error = crate::Error;

    async fn async_try_from(mut request: Request<Body>) -> Result<Self, Self::Error> {
        let chunks = request.body_mut().read_chunks().await?;

        let payload =
            serde_json::from_slice::<P>(&chunks).map_err(Error::JsonDeserializePayload)?;

        Ok(Wrap(payload))
    }
}
