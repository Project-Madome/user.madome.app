mod inmemory;
mod postgresql;
pub mod r#trait;

pub use inmemory::*;
pub use postgresql::*;

use std::sync::Arc;

use sai::{Component, ComponentLifecycle, Injected};

#[derive(Component)]
#[lifecycle]
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
}

impl RepositorySet {
    pub fn user(&self) -> Arc<impl r#trait::UserRepository> {
        Arc::clone(&self.user_repository)
    }

    pub fn like(&self) -> Arc<impl r#trait::LikeRepository> {
        Arc::clone(&self.like_repository)
    }
}

#[async_trait::async_trait]
impl ComponentLifecycle for RepositorySet {
    async fn start(&mut self) {}
}
