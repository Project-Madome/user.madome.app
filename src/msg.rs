use std::sync::Arc;

use hyper::{Body, Method, Request, Response};

use madome_sdk::api::{auth, cookie::MADOME_ACCESS_TOKEN, Token};
use util::{
    http::{
        url::{is_path_variable, PathVariable},
        Cookie,
    },
    BodyParser, ToPayload,
};
use uuid::Uuid;

use crate::{
    config::Config,
    usecase::{
        create_like, create_notifications, create_or_update_fcm_token, create_or_update_history,
        create_user, delete_history, delete_like, get_fcm_tokens, get_histories, get_histories_by,
        get_likes, get_likes_by, get_notifications, get_user,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
}

/// Msg의 Payload는 같은 이름의 usecase의 Payload와는 관계가 없음
///
/// Msg의 Payload는 실행되어야하는 usecase 순서에 따라 정해짐 (제일 처음 실행하는 usecase의 Payload)
///
/// 실행되는 순서는 Resolver 참조
#[derive(Debug)]
pub enum Msg {
    CreateUser(create_user::Payload),
    GetUser(get_user::Payload),

    CreateLike(create_like::Payload),
    GetLikes(get_likes::Payload),
    GetLikesBy(get_likes_by::Payload),
    DeleteLike(delete_like::Payload),

    CreateNotifications(create_notifications::Payload),
    GetNotifications(get_notifications::Payload),

    CreateOrUpdateFcmToken(create_or_update_fcm_token::Payload),
    GetFcmTokens(get_fcm_tokens::Payload),

    CreateOrUpdateHistory(create_or_update_history::Payload),
    GetHistories(get_histories::Payload),
    GetHistoriesBy(get_histories_by::Payload),
    DeleteHistory(delete_history::Payload),
}

impl Msg {
    pub async fn http(
        request: &mut Request<Body>,
        _resp: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<Self> {
        let headers = request.headers();

        /* let cookie = Cookie::from(headers);

        let access_token = cookie.get(MADOME_ACCESS_TOKEN).unwrap_or_default();
        let refresh_token = cookie.get(MADOME_REFRESH_TOKEN).unwrap_or_default(); */

        // let resp = RwLock::new(resp);

        let user_id = match auth::check_internal(headers) {
            Ok(_) => Uuid::nil(),
            Err(_) => {
                // 사용자 토큰 추가
                /* if let Some(cookie) = headers.get(header::COOKIE).cloned() {
                    let mut resp = resp.write();
                    resp.set_header(header::COOKIE, cookie).unwrap();
                } */

                let access_token = Cookie::from(headers)
                    .take(MADOME_ACCESS_TOKEN)
                    .unwrap_or_default();

                auth::check_access_token(config.auth_url(), Token::from(access_token), 0)
                    .await?
                    .user_id

                /* auth::check_and_refresh_token_pair(config.auth_url(), Token::Store(&resp), 0)
                .await?
                .user_id */
            }
        };

        let method = request.method().clone();
        let path = request.uri().path();
        let user_checked = !user_id.is_nil();

        let msg = match (method, path, user_checked) {
            /* Public */
            (Method::POST, "/users", true) => {
                let p = request.body_parse().await?;

                Msg::CreateUser(p)
            }

            /* Public */
            (Method::GET, "/users/@me", true) => {
                let p = get_user::Payload {
                    id_or_email: user_id.to_string(),
                };

                Msg::GetUser(p)
            }

            /* Public */
            (Method::POST, "/users/@me/likes", true) => {
                let p: create_like::Payload = request.to_payload(user_id).await?;

                Msg::CreateLike(p)
            }

            /* Public */
            (Method::GET, "/users/@me/likes", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::GetLikes(p)
            }

            /* Public */
            (Method::DELETE, "/users/@me/likes", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::DeleteLike(p)
            }

            /* Public */
            (Method::POST, "/users/@me/dislikes", true) => {
                todo!()
            }

            /* Public */
            (Method::GET, "/users/@me/dislikes", true) => {
                todo!()
            }

            /* Public */
            (Method::DELETE, "/users/@me/dislikes", true) => {
                todo!()
            }

            /* Public */
            (Method::GET, "/users/@me/notifications", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::GetNotifications(p)
            }

            /* Public */
            (Method::POST, "/users/@me/fcm-token", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::CreateOrUpdateFcmToken(p)
            }

            /* Public */
            (Method::POST, "/users/@me/histories", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::CreateOrUpdateHistory(p)
            }

            /* Public */
            (Method::DELETE, "/users/@me/histories", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::DeleteHistory(p)
            }

            /* Public */
            (Method::GET, "/users/@me/histories", true) => {
                let p = request.to_payload(user_id).await?;

                Msg::GetHistories(p)
            }

            /* Internal */
            (Method::POST, "/users/notifications", false) => {
                let p = request.body_parse().await?;

                Msg::CreateNotifications(p)
            }

            /* Internal */
            (Method::GET, "/users/fcm-token", false) => {
                let p = request.try_into()?;

                Msg::GetFcmTokens(p)
            }

            /* Internal */
            /* (Method::GET, "/users/likes", false) => {
                let p = request.to_payload(Uuid::nil()).await?;

                Msg::GetLikesBy(p)
            } */

            /* Internal */
            (Method::GET, path, false) if matcher(path, "/users/:user_id/likes") => {
                let user_id = PathVariable::from((path, "/users/:user_id/likes"))
                    .next_variable::<Uuid>()
                    .unwrap_or_default();

                let p = request.to_payload(user_id).await?;

                Msg::GetLikesBy(p)
            }

            (Method::GET, "/users/likes", false) => {
                todo!()
            }

            /* Internal */
            (Method::GET, path, false) if matcher(path, "/users/:user_id/dislikes") => {
                todo!()
            }

            /* Internal */
            /* (Method::GET, "/users/histories", false) => {
                let p = request.to_payload(Uuid::nil());

                todo!()
            } */

            /* Internal */
            (Method::GET, path, false) if matcher(path, "/users/:user_id/histories") => {
                let user_id = PathVariable::from((path, "/users/:user_id/histories"))
                    .next_variable::<Uuid>()
                    .unwrap_or_default();

                let p = request.to_payload(user_id).await?;

                Msg::GetHistoriesBy(p)
            }

            /* Internal */
            (Method::GET, path, false) if matcher(path, "/users/:user_id_or_email") => {
                let p: get_user::Payload =
                    PathVariable::from((path, "/users/:user_id_or_email")).into();

                Msg::GetUser(p)
            }

            _ => return Err(Error::NotFound.into()),
        };

        log::info!("{msg:?}");

        Ok(msg)
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

/*

pub struct Wrap<P>(pub P);

impl<P> Wrap<P> {
    pub fn inner(self) -> P {
        self.0
    }
}


#[async_trait::async_trait]
impl<'a, P> AsyncTryFrom<&'a mut Request<Body>> for Wrap<P>
where
    P: DeserializeOwned,
{
    type Error = crate::Error;

    async fn async_try_from(request: &'a mut Request<Body>) -> Result<Self, Self::Error> {
        let chunks = request.body_mut().read_chunks().await?;

        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|x| x.to_str().unwrap_or_default());

        match content_type {
            Some(content_type) if content_type.starts_with("application/json") => {
                let payload = serde_json::from_slice::<P>(&chunks)
                    .map_err(payload::Error::JsonDeserialize)?;

                Ok(Wrap(payload))
            }
            content_type => {
                let content_type = content_type.unwrap_or_default().to_owned();
                Err(payload::Error::NotSupportedContentType(content_type).into())
            }
        }
    }
} */
