mod user_repository;

use std::sync::Arc;

use sai::{Component, ComponentLifecycle, Injected};
pub use user_repository::InMemoryUserRepository;

pub mod r#trait {
    pub use super::user_repository::r#trait::UserRepository;
}

#[derive(Component)]
#[lifecycle]
pub struct RepositorySet {
    #[injected]
    user_repository: Injected<InMemoryUserRepository>,
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
