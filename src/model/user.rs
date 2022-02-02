use chrono::{DateTime, Utc};
use hyper::{header, StatusCode};
use serde::Serialize;

use crate::entity;

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

impl Presenter for User {
    fn to_http(self, response: hyper::http::response::Builder) -> hyper::Response<hyper::Body> {
        let serialized = serde_json::to_vec(&self).expect("json serialize");

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
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
