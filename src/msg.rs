use hyper::{header, http::response::Builder as ResponseBuilder, Body, Method, Request};
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
use uuid::Uuid;

use crate::{
    payload,
    usecase::{
        create_like, create_notifications, create_or_update_fcm_token, create_user, delete_like,
        get_fcm_tokens, get_likes, get_likes_from_book_tags, get_notifications, get_user,
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

    CreateNotifications(create_notifications::Payload),
    GetNotifications(get_notifications::Payload),

    CreateOrUpdateFcmToken(create_or_update_fcm_token::Payload),
    GetFcmTokens(get_fcm_tokens::Payload),
}

impl Msg {
    pub async fn http(
        request: Request<Body>,
        mut response: ResponseBuilder,
        madome_auth_url: String,
    ) -> Result<(Self, ResponseBuilder), (crate::Error, ResponseBuilder)> {
        use Role::*;

        let headers = request.headers();
        let cookie = Cookie::from(headers);

        let access_token = cookie.get(MADOME_ACCESS_TOKEN).unwrap_or_default();
        let refresh_token = cookie.get(MADOME_REFRESH_TOKEN).unwrap_or_default();

        let auth = Auth::new(&madome_auth_url);

        let user_id = match auth.check_internal(headers).is_ok() {
            true => Uuid::nil(),
            false => {
                let r = auth
                    .check_and_refresh_token_pair(access_token, refresh_token, Developer)
                    .await;

                let (r, token_pair) = match r {
                    Ok(r) => r,
                    Err(err) => return Err((err.into(), response)),
                };

                response = response.headers(token_pair.iter());

                r.user_id
            }
        };

        // do not mutate response in this async block
        let msg: crate::Result<Msg> = async move {
            let method = request.method().clone();
            let path = request.uri().path();
            let user_checked = !user_id.is_nil();

            match (method, path, user_checked) {
                /* Public */
                (Method::POST, "/users", true) => {
                    let p = Wrap::async_try_from(request).await?.inner();

                    Ok(Msg::CreateUser(p))
                }

                (Method::GET, "/users/@me", true) => {
                    let p = get_user::Payload {
                        id_or_email: user_id.to_string(),
                    };

                    Ok(Msg::GetUser(p))
                }

                (Method::POST, "/users/@me/likes", true) => {
                    let mut p: create_like::Payload = Wrap::async_try_from(request).await?.inner();
                    p.set_user_id(user_id);

                    Ok(Msg::CreateLike(p))
                }

                (Method::GET, "/users/@me/likes", true) => {
                    let p = request.into_payload(user_id).await?;

                    Ok(Msg::GetLikes(p))
                }

                (Method::DELETE, "/users/@me/likes", true) => {
                    let p = request.into_payload(user_id).await?;

                    Ok(Msg::DeleteLike(p))
                }

                (Method::GET, "/users/@me/notifications", true) => {
                    let p = request.into_payload(user_id).await?;

                    Ok(Msg::GetNotifications(p))
                }

                (Method::POST, "/users/@me/fcm-token", true) => {
                    let p = request.into_payload(user_id).await?;

                    Ok(Msg::CreateOrUpdateFcmToken(p))
                }

                /* Internal */
                (Method::GET, "/users/likes/book-tags", false) => {
                    let p = request.try_into()?;

                    Ok(Msg::GetLikesFromBookTags(p))
                }

                (Method::POST, "/users/notifications", false) => {
                    let p = Wrap::async_try_from(request).await?.inner();

                    Ok(Msg::CreateNotifications(p))
                }

                (Method::GET, "/users/fcm-token", false) => {
                    let p = request.try_into()?;

                    Ok(Msg::GetFcmTokens(p))
                }

                (Method::GET, path, false) if matcher(path, "/users/:user_id_or_email") => {
                    let p: get_user::Payload =
                        PathVariable::from((path, "/users/:user_id_or_email")).into();

                    Ok(Msg::GetUser(p))
                }

                _ => Err(Error::NotFound.into()),
            }
        }
        .await;

        match msg {
            Ok(msg) => Ok((msg, response)),
            Err(err) => Err((err, response)),
        }
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

        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|x| x.to_str().unwrap_or_default());

        match content_type {
            Some(content_type) if content_type.starts_with("application/json") => {
                let payload =
                    serde_json::from_slice::<P>(&chunks).map_err(Error::JsonDeserializePayload)?;

                Ok(Wrap(payload))
            }
            content_type => {
                let content_type = content_type.unwrap_or_default().to_owned();
                Err(payload::Error::NotSupportedContentType(content_type).into())
            }
        }
    }
}
