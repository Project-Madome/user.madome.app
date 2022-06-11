use uuid::Uuid;

use crate::entity::{History, HistoryKind, HistorySortBy};

pub enum HistoryBy {
    Book { ids: Vec<u32> },
}

#[async_trait::async_trait]
pub trait HistoryRepository: Send + Sync {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<HistoryKind>,
        per_page: usize,
        page: usize,
        sort_by: HistorySortBy,
    ) -> crate::Result<Vec<History>>;

    async fn get_many_by(&self, user_id: Uuid, by: HistoryBy) -> crate::Result<Vec<History>>;

    async fn add_or_update(&self, history: History) -> crate::Result<()>;

    // async fn update(&self, history: History) -> crate::Result<Option<History>>;

    async fn remove(&self, history: History) -> crate::Result<bool>;
}
