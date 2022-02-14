use serde::Deserialize;

use crate::entity;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LikeKind {
    Book,
    BookTag,
}

impl From<LikeKind> for entity::LikeKind {
    fn from(kind: LikeKind) -> Self {
        use entity::LikeKind::*;

        match kind {
            LikeKind::Book => Book,
            LikeKind::BookTag => BookTag,
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LikeSortBy {
    CreatedAtDesc,
    CreatedAtAsc,
    Random,
}

impl From<LikeSortBy> for entity::LikeSortBy {
    fn from(sort_by: LikeSortBy) -> Self {
        use entity::LikeSortBy::*;
        use entity::Sort::*;

        match sort_by {
            LikeSortBy::CreatedAtDesc => CreatedAt(Desc),
            LikeSortBy::CreatedAtAsc => CreatedAt(Asc),
            LikeSortBy::Random => Random,
        }
    }
}
