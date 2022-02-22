use std::sync::Arc;

use chrono::{DateTime, Utc};
use hyper::{header, Body, Request, Response, StatusCode};
use serde::Serialize;
use util::http::SetResponse;

use crate::{config::Config, entity};

use super::Presenter;

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
impl Presenter for User {
    async fn set_response(
        self,
        _request: &mut Request<Body>,
        resp: &mut Response<Body>,
        _config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = serde_json::to_vec(&self).expect("json serialize");

        resp.set_status(StatusCode::OK).unwrap();
        resp.set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        resp.set_body(serialized.into());

        Ok(())
    }
}

impl From<entity::User> for User {
    fn from(
        entity::User {
            id,
            email,
            name,
            role,
            created_at,
            updated_at,
        }: entity::User,
    ) -> Self {
        Self {
            id: id.to_string(),
            email,
            name,
            role: role.into(),
            created_at,
            updated_at,
        }
    }
}

impl From<entity::UserRole> for u8 {
    fn from(role: entity::UserRole) -> Self {
        match role {
            entity::UserRole::Normal => 0,
            entity::UserRole::Developer => 1,
        }
    }
}
