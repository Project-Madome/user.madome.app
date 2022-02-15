use uuid::Uuid;

use crate::entity::fcm_token::FcmToken;

#[async_trait::async_trait]
pub trait FcmTokenRepository: Send + Sync {
    async fn add_or_update(&self, fcm_token: FcmToken) -> crate::Result<()>;

    async fn get_many(&self, user_ids: Vec<Uuid>) -> crate::Result<Vec<String>>;
}
