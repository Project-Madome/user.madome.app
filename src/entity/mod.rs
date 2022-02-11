pub mod like;
pub mod user;

pub use like::{Like, LikeKind};
pub use user::{User, UserRole};

#[derive(Debug, Clone, Copy)]
pub enum Sort {
    Desc,
    Asc,
}
