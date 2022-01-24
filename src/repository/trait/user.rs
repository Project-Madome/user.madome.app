use crate::entity::user::User;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    // async fn update(&self, ) -> crate::Result<Option<User>>;

    async fn add(&self, user: User) -> crate::Result<Option<User>>;

    async fn get(&self, id_or_email: String) -> crate::Result<Option<User>>;
}
