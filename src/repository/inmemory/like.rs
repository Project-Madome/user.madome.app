use std::{collections::HashMap, sync::RwLock};

use itertools::Itertools;
use sai::Component;
use uuid::Uuid;

use crate::{
    entity::{like::LikeSortBy, Like, LikeKind, Sort},
    repository::r#trait::LikeRepository,
};

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct InMemoryLikeRepository {
    inner: RwLock<HashMap<Uuid, Vec<Like>>>,
}

#[async_trait::async_trait]
impl LikeRepository for InMemoryLikeRepository {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<LikeKind>,
        offset: usize,
        page: usize,
        sort_by: LikeSortBy,
    ) -> crate::Result<Vec<Like>> {
        let inner = self.inner.read().unwrap();

        let filter_by_kind = |x: &&Like| -> bool {
            match kind {
                Some(LikeKind::Book) => matches!(x.kind(), LikeKind::Book),
                Some(LikeKind::BookTag) => matches!(x.kind(), LikeKind::BookTag),
                None => true,
            }
        };

        match inner.get(&user_id) {
            Some(r) => {
                let r = r.iter().filter(filter_by_kind);

                let r = match sort_by {
                    LikeSortBy::CreatedAt(Sort::Desc) => {
                        r.sorted_by(|a, b| b.created_at().cmp(&a.created_at()))
                    }
                    LikeSortBy::CreatedAt(Sort::Asc) => {
                        r.sorted_by(|a, b| a.created_at().cmp(&b.created_at()))
                    }
                    LikeSortBy::Random => unimplemented!(
                        "unimplemented sort by random in InMemoryLikeRepository::get_many()"
                    ),
                }
                .skip(offset * (page - 1))
                .take(offset)
                .cloned()
                .collect();

                Ok(r)
            }
            None => Ok(Vec::new()),
        }
    }

    // TODO: 일단 정렬안함
    async fn get_many_by_book_tags(
        &self,
        book_tags: Vec<(String, String)>,
    ) -> crate::Result<Vec<Like>> {
        let inner = self.inner.read().unwrap();

        let r = inner
            .values()
            .flatten()
            .filter(|x| {
                matches!(x, Like::BookTag { tag_kind, tag_name, .. }
                    if book_tags.contains(&(tag_kind.to_owned(), tag_name.to_owned())))
            })
            .cloned()
            .collect();

        Ok(r)
    }

    async fn add(&self, like: Like) -> crate::Result<bool> {
        let mut inner = self.inner.write().unwrap();

        let user_id = like.user_id();

        let likes = inner.get(&user_id);

        let already_exists = match (&like, likes) {
            (Like::Book { book_id, .. }, Some(likes)) => likes.iter().any(|x| {
                matches!(x, Like::Book { book_id: exists, .. }
                    if exists == book_id)
            }),

            (
                Like::BookTag {
                    tag_kind, tag_name, ..
                },
                Some(likes),
            ) => {
                let exists = (tag_kind, tag_name);

                likes.iter().any(|x| {
                    matches!(x, Like::BookTag { tag_kind, tag_name, .. }
                        if exists == (tag_kind, tag_name))
                })
            }
            _ => false,
        };

        if already_exists {
            return Ok(false);
        }

        let likes = inner.get_mut(&user_id);

        match likes {
            Some(likes) => {
                likes.push(like);
            }

            None => {
                inner.insert(user_id.to_owned(), vec![like]);
            }
        }

        Ok(true)
    }

    async fn remove(&self, like: Like) -> crate::Result<bool> {
        let mut inner = self.inner.write().unwrap();

        let user_id = like.user_id();

        let likes = inner.get(&user_id);

        let position = match (&like, likes) {
            // Book
            (Like::Book { book_id, .. }, Some(likes)) => likes
                .iter()
                .enumerate()
                .find(|(_, x)| {
                    matches!(x, Like::Book { book_id: exists, .. }
                        if exists == book_id)
                })
                .map(|(i, _)| i),

            // BookTag
            (
                Like::BookTag {
                    tag_kind, tag_name, ..
                },
                Some(likes),
            ) => likes
                .iter()
                .enumerate()
                .find(|(_, x)| {
                    matches!(x, Like::BookTag { tag_kind: exi, tag_name: sts, .. }
                        if (exi, sts) == (tag_kind, tag_name))
                })
                .map(|(i, _)| i),
            _ => None,
        };

        match position {
            Some(position) => {
                let likes = inner.get_mut(&user_id).unwrap();

                likes.remove(position);

                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[cfg(test)]
pub mod get_many {
    use chrono::Utc;
    use itertools::Itertools;
    use rand::{prelude::SliceRandom, thread_rng};
    use sai::{Component, System};
    use util::test_registry;
    use uuid::Uuid;

    use crate::{
        entity::{like::LikeSortBy, Like, LikeKind, Sort},
        repository::r#trait::LikeRepository,
    };

    use super::InMemoryLikeRepository;

    #[tokio::test]
    async fn success() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: InMemoryLikeRepository] ->
        [likes: Vec<Like>, user_id: Uuid, another_user_id: Uuid] ->
        {
            user_id = Uuid::new_v4();
            another_user_id = Uuid::new_v4();

            likes.append(
                &mut (0..25)
                    .into_iter()
                    .map(|x| Like::Book {
                        user_id,
                        book_id: x,
                        created_at: Utc::now()
                    })
                    .collect()
            );

            // push another user_id
            {
                likes.append(
                    &mut (0..25)
                        .into_iter()
                        .map(|x| Like::Book {
                            user_id: another_user_id,
                            book_id: x,
                            created_at: Utc::now()
                        })
                        .collect()
                );

                likes.append(
                    &mut (25..50)
                        .into_iter()
                        .map(|x| Like::BookTag {
                            user_id: another_user_id,
                            tag_kind: (x * x).to_string(),
                            tag_name: (x * x * x).to_string(),
                            created_at: Utc::now()
                        })
                        .collect()
                );
            }

            likes.append(
                &mut (25..50)
                    .into_iter()
                    .map(|x| Like::BookTag {
                        user_id,
                        tag_kind: (x * x).to_string(),
                        tag_name: (x * x * x).to_string(),
                        created_at: Utc::now()
                    })
                    .collect()
            );

            likes.shuffle(&mut thread_rng());

            for like in likes.iter() {
                let saved = repository.add(like.clone()).await.unwrap();

                if !saved {
                    panic!("not saved like: {like:?}");
                }
            }
        },
        {
            // Book
            let r = repository.get_many(user_id, Some(LikeKind::Book), 25, 1, LikeSortBy::CreatedAt(Sort::Desc)).await.unwrap();
            let e = likes
                .iter()
                .filter(|x| x.user_id() == user_id)
                .filter(|x| x.kind() == LikeKind::Book)
                .sorted_by(|a, b| b.created_at().cmp(&a.created_at()))
                .take(25)
                .cloned()
                .collect::<Vec<_>>();

            assert_eq!(r, e, "{user_id}");

            // BookTag
            let r = repository.get_many(user_id, Some(LikeKind::BookTag), 25, 1, LikeSortBy::CreatedAt(Sort::Desc)).await.unwrap();
            let e = likes
                .iter()
                .filter(|x| x.user_id() == user_id)
                .filter(|x| x.kind() == LikeKind::BookTag)
                .sorted_by(|a, b| b.created_at().cmp(&a.created_at()))
                .take(25)
                .cloned()
                .collect::<Vec<_>>();

            assert_eq!(r, e);

            // All
            let r = repository.get_many(user_id, None, 75, 1, LikeSortBy::CreatedAt(Sort::Desc)).await.unwrap();
            let e = likes
                .iter()
                .filter(|x| x.user_id() == user_id)
                .sorted_by(|a, b| b.created_at().cmp(&a.created_at()))
                .take(75)
                .cloned()
                .collect::<Vec<_>>();

            assert_eq!(r, e);
        });
    }
}

#[cfg(test)]
pub mod get_many_by_book_tags {
    use chrono::Utc;
    use rand::{prelude::SliceRandom, thread_rng};
    use sai::{Component, System};
    use util::test_registry;
    use uuid::Uuid;

    use crate::{entity::Like, repository::r#trait::LikeRepository};

    use super::InMemoryLikeRepository;

    #[tokio::test]
    async fn success() {
        let mut test = System::<TestRegistry>::new();

        test.start().await;

        test_registry!(
        [repository: InMemoryLikeRepository] ->
        [user_id: Uuid, likes: Vec<Like>] ->
        {
            user_id = Uuid::new_v4();

            likes.append(
                &mut (0..30)
                    .map(|x| Like::BookTag {
                        user_id,
                        tag_kind: (x * x).to_string(),
                        tag_name: (x * x * x).to_string(),
                        created_at: Utc::now()
                    })
                    .collect()
            );

            likes.shuffle(&mut thread_rng());

            for like in likes.iter() {
                let saved = repository.add(like.clone()).await.unwrap();

                if !saved {
                    panic!("not saved like: {like:?}");
                }
            }
        },
        {
            let book_tags = (10..25)
                .map(|x| ((x * x).to_string(), (x * x * x).to_string()))
                .collect::<Vec<_>>();

            let r = repository.get_many_by_book_tags(book_tags.clone()).await.unwrap();
            let e = likes
                .into_iter()
                .filter(|x| {
                    matches!(x, Like::BookTag { tag_kind, tag_name , .. }
                        if book_tags.contains(&(tag_kind.to_owned(), tag_name.to_owned())))
                })
                .collect::<Vec<_>>();

            assert_eq!(r, e);
        });
    }
}
