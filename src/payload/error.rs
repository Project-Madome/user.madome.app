use util::validate::number;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum Error {
    #[error("Not supported content-type: {0}")]
    NotSupportedContentType(String),

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
