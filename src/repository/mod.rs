mod inmemory;
mod postgresql;
pub mod r#trait;

pub use inmemory::*;
pub use postgresql::*;

use std::sync::Arc;

use sai::{Component, Injected};

#[derive(Component)]
pub struct RepositorySet {
    #[cfg(not(test))]
    #[injected]
    user_repository: Injected<PostgresqlUserRepository>,

    #[cfg(test)]
    #[injected]
    user_repository: Injected<InMemoryUserRepository>,

    #[cfg(not(test))]
    #[injected]
    like_repository: Injected<PostgresqlLikeRepository>,

    #[cfg(test)]
    #[injected]
    like_repository: Injected<InMemoryLikeRepository>,

    #[injected]
    notification_repository: Injected<PostgresqlNotificationRepository>,
}

impl RepositorySet {
    pub fn user(&self) -> Arc<impl r#trait::UserRepository> {
        Arc::clone(&self.user_repository)
    }

    pub fn like(&self) -> Arc<impl r#trait::LikeRepository> {
        Arc::clone(&self.like_repository)
    }

    pub fn notification(&self) -> Arc<impl r#trait::NotificationRepository> {
        Arc::clone(&self.notification_repository)
    }
}
