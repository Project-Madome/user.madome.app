use serde::Deserialize;
use std::sync::Arc;

use crate::{
    entity::{Notification, NotificationKind},
    error::UseCaseError,
    payload::notification::NotificationBook,
    repository::{r#trait::NotificationRepository, RepositorySet},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "content", rename_all = "snake_case")]
pub enum Payload {
    Book(Vec<NotificationBook>),
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

pub async fn execute(p: Payload, repository: Arc<RepositorySet>) -> crate::Result<Model> {
    match p {
        Payload::Book(notifications) => {
            let notifications = notifications
                .into_iter()
                .map(|x| Notification::book(x.user_id, x.book_id, x.book_tags))
                .collect();

            let _r = repository
                .notification()
                .add_many(NotificationKind::Book, notifications)
                .await?;

            Ok(Model)
        }
    }
}

#[cfg(test)]
mod payload_tests {
    use std::str::FromStr;

    use uuid::Uuid;

    use crate::payload::notification::NotificationBook;

    use super::Payload;

    #[tokio::test]
    async fn inject_book() {
        let input = r#"
            {
                "kind": "book",
                "content": [
                    {
                        "book_id": 123456,
                        "book_tags": [["female", "loli"], ["female", "rape"]],
                        "user_id": "c5d494ff-31ce-4706-a2b5-eb9744d67ec9"
                    },
                    {
                        "book_id": 123452,
                        "book_tags": [["female", "large insertions"], ["female", "anal"]],
                        "user_id": "e01b116d-d96b-4d48-b491-e0fee71c1aa7"
                    }
                ]
            }"#;

        let payload: Payload = serde_json::from_str(input).unwrap();

        let expected = Payload::Book(vec![
            NotificationBook {
                book_id: 123456,
                book_tags: vec![
                    ("female".to_string(), "loli".to_string()),
                    ("female".to_string(), "rape".to_string()),
                ],
                user_id: Uuid::from_str("c5d494ff-31ce-4706-a2b5-eb9744d67ec9").unwrap(),
            },
            NotificationBook {
                book_id: 123452,
                book_tags: vec![
                    ("female".to_string(), "large insertions".to_string()),
                    ("female".to_string(), "anal".to_string()),
                ],
                user_id: Uuid::from_str("e01b116d-d96b-4d48-b491-e0fee71c1aa7").unwrap(),
            },
        ]);

        assert_eq!(payload, expected);
    }
}
