use uuid::Uuid;

use crate::entity::{like::LikeSortBy, Like, LikeKind};

#[async_trait::async_trait]
pub trait LikeRepository: Send + Sync {
    // TODO: sort_by
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<LikeKind>,
        offset: usize,
        page: usize,
        sort_by: LikeSortBy,
    ) -> crate::Result<Vec<Like>>;

    async fn get_many_by_book_tags(
        &self,
        book_tags: Vec<(String, String)>,
    ) -> crate::Result<Vec<Like>>;

    async fn add(&self, like: Like) -> crate::Result<bool>;

    async fn remove(&self, like: Like) -> crate::Result<bool>;
}
