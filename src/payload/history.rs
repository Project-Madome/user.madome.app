use serde::Deserialize;

use crate::entity;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HistoryKind {
    Book,
}

impl From<HistoryKind> for entity::HistoryKind {
    fn from(kind: HistoryKind) -> Self {
        use entity::HistoryKind::*;

        match kind {
            HistoryKind::Book => Book,
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HistorySortBy {
    CreatedAtDesc,
    CreatedAtAsc,
    UpdatedAtDesc,
    UpdatedAtAsc,
    Random,
}

impl From<HistorySortBy> for entity::HistorySortBy {
    fn from(sort_by: HistorySortBy) -> Self {
        use entity::HistorySortBy::*;
        use entity::Sort::*;

        match sort_by {
            HistorySortBy::CreatedAtDesc => CreatedAt(Desc),
            HistorySortBy::CreatedAtAsc => CreatedAt(Asc),
            HistorySortBy::UpdatedAtDesc => UpdatedAt(Desc),
            HistorySortBy::UpdatedAtAsc => UpdatedAt(Asc),
            HistorySortBy::Random => Random,
        }
    }
}
