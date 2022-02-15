use util::validate::number;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("json deserialize: {0}")]
    JsonDeserialize(serde_json::Error),
    #[error("querystring deserialize: {0}")]
    QuerystringDeserialize(serde_qs::Error),

    #[error("per-page: {0}")]
    InvalidPerPage(number::Error<usize>),
    #[error("page: {0}")]
    InvalidPage(number::Error<usize>),
    #[error("sort-by: {0}")]
    InvalidSortBy(String),
}
