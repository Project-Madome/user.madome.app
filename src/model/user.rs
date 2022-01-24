use chrono::{DateTime, Utc};
use hyper::{Body, Response, StatusCode};
use serde::Serialize;

use crate::{constant::http::header, entity};

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for Response<Body> {
    fn from(user: User) -> Self {
        let serialized = serde_json::to_vec(&user).expect("json serialize");

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, header::APPLICATION_JSON)
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
