use uuid::Uuid;

use crate::entity::{Like, LikeKind, LikeSortBy};

pub enum LikeBy {
    Book { ids: Vec<u32> },
    BookTag { tags: Vec<(String, String)> },
}

impl LikeBy {}

#[async_trait::async_trait]
pub trait LikeRepository: Send + Sync {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<LikeKind>,
        per_page: usize,
        page: usize,
        sort_by: LikeSortBy,
    ) -> crate::Result<Vec<Like>>;

    async fn get_many_by(&self, user_id: Option<Uuid>, by: LikeBy) -> crate::Result<Vec<Like>>;

    /* async fn get_many_by_book_tags(
        &self,
        book_tags: Vec<(String, String)>,
    ) -> crate::Result<Vec<Like>>; */

    async fn add(&self, like: Like) -> crate::Result<bool>;

    async fn remove(&self, like: Like) -> crate::Result<bool>;
}
