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
    #[injected]
    user_repository: Injected<PostgresqlUserRepository>,
}

impl RepositorySet {
    pub fn user(&self) -> Arc<impl r#trait::UserRepository> {
        Arc::clone(&self.user_repository)
    }
}

#[async_trait::async_trait]
impl ComponentLifecycle for RepositorySet {
    async fn start(&mut self) {}
}
