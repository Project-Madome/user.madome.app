use serde::Deserialize;
use uuid::Uuid;

use crate::entity;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationKind {
    Book,
}

impl From<NotificationKind> for entity::NotificationKind {
    fn from(kind: NotificationKind) -> Self {
        use entity::NotificationKind::*;

        match kind {
            NotificationKind::Book => Book,
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationSortBy {
    CreatedAtDesc,
    CreatedAtAsc,
}

impl From<NotificationSortBy> for entity::NotificationSortBy {
    fn from(sort_by: NotificationSortBy) -> Self {
        use entity::NotificationSortBy::*;
        use entity::Sort::*;

        match sort_by {
            NotificationSortBy::CreatedAtDesc => CreatedAt(Desc),
            NotificationSortBy::CreatedAtAsc => CreatedAt(Asc),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
pub struct NotificationBook {
    pub book_id: u32,
    pub book_tags: Vec<(String, String)>,
    pub user_id: Uuid,
}
