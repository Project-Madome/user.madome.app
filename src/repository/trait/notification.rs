use uuid::Uuid;

use crate::entity::{notification::NotificationSortBy, Notification, NotificationKind};

#[async_trait::async_trait]
pub trait NotificationRepository {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<NotificationKind>,
        offset: usize,
        page: usize,
        sort_by: NotificationSortBy,
    ) -> crate::Result<Vec<Notification>>;

    // async fn add(&self, noti: Notification) -> crate::Result<bool>;

    async fn add_many(
        &self,
        kind: NotificationKind,
        notifications: Vec<Notification>,
    ) -> crate::Result<()>;
}
