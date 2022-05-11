use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};

use crate::{
    command::CommandSet,
    entity::{Notification, NotificationKind},
    error::UseCaseError,
    model::Like,
    repository::{r#trait::NotificationRepository, RepositorySet},
    usecase::get_likes_from_book_tags,
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    Book {
        book_id: u32,
        book_title: String,
        book_tags: Vec<(String, String)>,
    },
}

pub struct Model;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        UseCaseError::from(err).into()
    }
}

/// Book: Library 서버에서 작품이 업로드될때마다 해당 api에 작품 id와 타이틀, 태그를 전달함
pub async fn execute(
    p: Payload,
    repository: Arc<RepositorySet>,
    #[allow(unused_variables)] command: Arc<CommandSet>,
) -> crate::Result<Model> {
    match p {
        Payload::Book {
            book_tags,
            book_id,
            book_title,
        } => {
            #[allow(unused_variables)]
            let book_title = book_title;

            let p = get_likes_from_book_tags::Payload { book_tags };
            let likes = get_likes_from_book_tags::execute(p, repository.clone()).await?;

            // group by user id and book id
            let group_by = likes.into_iter().fold(BTreeMap::new(), |mut acc, like| {
                if let Like::BookTag {
                    user_id,
                    tag_kind,
                    tag_name,
                    ..
                } = like
                {
                    let tag = (tag_kind, tag_name);

                    let r = acc.entry((user_id, book_id)).or_insert_with(Vec::new);

                    r.push(tag);
                }

                acc
            });

            let notifications = group_by
                .into_iter()
                .map(|((user_id, book_id), book_tags)| {
                    Notification::book(user_id, book_id, book_tags)
                })
                .collect::<Vec<_>>();

            let _r = repository
                .notification()
                .add_many(NotificationKind::Book, notifications.clone())
                .await?;

            #[cfg(feature = "fcm")]
            {
                use crate::{command::send_notification::Message, usecase::get_fcm_tokens};

                let user_ids = notifications.iter().map(|x| x.user_id()).collect();

                let p = get_fcm_tokens::Payload { user_ids };
                let fcm_tokens = get_fcm_tokens::execute(p, repository.clone())
                    .await
                    .map(|x| x.0);

                if let Ok(fcm_tokens) = fcm_tokens {
                    // TODO: error handle : command::send_notification
                    command
                        .send_notification(
                            fcm_tokens,
                            Message::new("좋아하실만한 작품이 올라왔어요.", book_title),
                        )
                        .await
                        .ok();
                }
            }

            Ok(Model)
        }
    }
}

#[cfg(test)]
mod payload_tests {
    // use crate::payload::notification::NotificationBook;

    use super::Payload;

    #[tokio::test]
    async fn inject_book() {
        /* let input = r#"
            {
                "kind": "book",
                "content": [
                    {
                        "book_id": 123456,
                        "book_tags": [["female", "loli"], ["female", "rape"]]
                    },
                    {
                        "book_id": 123452,
                        "book_tags": [["female", "large insertions"], ["female", "anal"]]
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
                // user_id: Uuid::from_str("c5d494ff-31ce-4706-a2b5-eb9744d67ec9").unwrap(),
            },
            NotificationBook {
                book_id: 123452,
                book_tags: vec![
                    ("female".to_string(), "large insertions".to_string()),
                    ("female".to_string(), "anal".to_string()),
                ],
                // user_id: Uuid::from_str("e01b116d-d96b-4d48-b491-e0fee71c1aa7").unwrap(),
            },
        ]); */

        let input = r#"
            {
                "kind": "book",
                "book_id": 123456,
                "book_title": "COMIC-LO",
                "book_tags": [["female", "loli"], ["female", "rape"]]
                    
            }"#;

        let payload: Payload = serde_json::from_str(input).unwrap();

        let expected = Payload::Book {
            book_id: 123456,
            book_title: "COMIC-LO".to_string(),
            book_tags: vec![
                ("female".to_string(), "loli".to_string()),
                ("female".to_string(), "rape".to_string()),
            ],
        };

        assert_eq!(payload, expected);
    }
}
