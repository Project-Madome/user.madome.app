pub mod like;
pub mod notification;
pub mod user;

pub use like::{Like, LikeKind, LikeSortBy};
pub use notification::{Notification, NotificationKind, NotificationSortBy};
pub use user::{User, UserRole};

#[derive(Debug, Clone, Copy)]
pub enum Sort {
    Desc,
    Asc,
}
