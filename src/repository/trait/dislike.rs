use uuid::Uuid;

use crate::entity::{Dislike, DislikeKind, DislikeSortBy};

#[async_trait::async_trait]
pub trait DislikeRepository: Send + Sync {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<DislikeKind>,
        per_page: usize,
        page: usize,
        sort_by: DislikeSortBy,
    ) -> crate::Result<Vec<Dislike>>;

    async fn add(&self, dislike: Dislike) -> crate::Result<bool>;

    async fn remove(&self, dislike: Dislike) -> crate::Result<bool>;
}
