use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, IdenStatic,
    PaginatorTrait, QueryFilter, QueryOrder, Statement, TryGetable,
};
use util::sea_orm::OrderByRandom;
use uuid::Uuid;

use crate::{
    constant::postgresql,
    database::{postgresql::entity::like, DatabaseSet},
    entity::{like::LikeSortBy, Like, LikeKind, Sort},
    repository::r#trait::LikeRepository,
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlLikeRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlLikeRepository {
    async fn start(&mut self) {
        like::book::create_table(self.database.postgresql()).await;
        like::book_tag::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl LikeRepository for PostgresqlLikeRepository {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<LikeKind>,
        offset: usize,
        page: usize,
        sort_by: LikeSortBy,
        // TODO: enum SortBy { CreatedAt, Random }
    ) -> crate::Result<Vec<Like>> {
        let likes = match kind {
            Some(LikeKind::Book) => {
                let select = like::book::Entity::find();
                let r = match sort_by {
                    LikeSortBy::Random => select.order_by_random(),
                    LikeSortBy::CreatedAt(Sort::Desc) => {
                        select.order_by_desc(like::book::Column::CreatedAt)
                    }
                    LikeSortBy::CreatedAt(Sort::Asc) => {
                        select.order_by_asc(like::book::Column::CreatedAt)
                    }
                }
                .filter(like::book::Column::UserId.eq(user_id))
                /* .query()
                .offset(offset * (page - 1))
                .limit(offset) */
                .paginate(self.database.postgresql(), offset) // case1
                .fetch_page(page - 1) // case2
                .await?;

                r.into_iter().map(Into::into).collect()
            }

            Some(LikeKind::BookTag) => {
                let select = like::book_tag::Entity::find();
                let r = match sort_by {
                    LikeSortBy::Random => select.order_by_random(),
                    LikeSortBy::CreatedAt(Sort::Desc) => {
                        select.order_by_desc(like::book_tag::Column::CreatedAt)
                    }
                    LikeSortBy::CreatedAt(Sort::Asc) => {
                        select.order_by_asc(like::book_tag::Column::CreatedAt)
                    }
                }
                .order_by_desc(like::book_tag::Column::CreatedAt)
                .filter(like::book_tag::Column::UserId.eq(user_id))
                .paginate(self.database.postgresql(), offset)
                .fetch_page(page - 1)
                .await?;

                r.into_iter().map(Into::into).collect()
            }

            None => {
                let sort_by = match sort_by {
                    LikeSortBy::CreatedAt(Sort::Desc) => "created_at DESC",
                    LikeSortBy::CreatedAt(Sort::Asc) => "created_at ASC",
                    LikeSortBy::Random => "RANDOM() DESC",
                };

                let query = format!(
                    r#"
                SELECT * FROM
                (
                    SELECT id, user_id, book_id, NULL AS tag_kind, NULL AS tag_name, created_at
                        FROM likes_book
                        WHERE user_id = $1
                    UNION ALL
                    SELECT id, user_id, NULL, tag_kind, tag_name, created_at
                        FROM likes_book_tag
                        WHERE user_id = $1
                ) AS a
                ORDER BY {sort_by}"#,
                );

                let db = self.database.postgresql();
                let psql = db.get_database_backend();

                let query_results = db
                    .query_all(Statement::from_sql_and_values(
                        psql,
                        &query,
                        [user_id.into()],
                    ))
                    .await?;

                query_results
                    .iter()
                    .map(|x| {
                        // partitioning by LikeKind
                        let like_kind = {
                            // TODO: if another kind added, change kind partitioning logic
                            let r =
                                Option::<i32>::try_get(x, "", like::book::Column::BookId.as_str())
                                    .map_err(DbErr::from)?;

                            log::debug!("{r:?}");

                            match r {
                                Some(_) => LikeKind::Book,
                                None => LikeKind::BookTag,
                            }
                        };

                        // into Like
                        match like_kind {
                            LikeKind::Book => {
                                let model = like::book::Model::from_query_result(x, "")?;

                                Ok(Like::from(model))
                            }
                            LikeKind::BookTag => {
                                let model = like::book_tag::Model::from_query_result(x, "")?;

                                Ok(Like::from(model))
                            }
                        }
                    })
                    .collect::<Result<_, DbErr>>()?
            }
        };

        Ok(likes)
    }

    // TODO: 일단 정렬안함
    async fn get_many_by_book_tags(
        &self,
        book_tags: Vec<(String, String)>,
    ) -> crate::Result<Vec<Like>> {
        if book_tags.is_empty() {
            return Ok(Vec::new());
        }

        let cond = book_tags
            .into_iter()
            .fold(Condition::any(), |acc, (kind, name)| {
                acc.add(
                    Condition::all()
                        .add(like::book_tag::Column::TagKind.eq(kind))
                        .add(like::book_tag::Column::TagName.eq(name)),
                )
            });

        let likes = like::book_tag::Entity::find()
            .filter(Condition::all().add(cond))
            .all(self.database.postgresql())
            .await?;

        Ok(likes.into_iter().map(Into::into).collect())
    }

    async fn add(&self, like: Like) -> crate::Result<bool> {
        let r = match like.kind() {
            LikeKind::Book => {
                let r = like::book::Entity::insert::<like::book::ActiveModel>(like.into())
                    .exec(self.database.postgresql())
                    .await;

                r.map(|_x| ())
            }

            LikeKind::BookTag => {
                let r = like::book_tag::Entity::insert::<like::book_tag::ActiveModel>(like.into())
                    .exec(self.database.postgresql())
                    .await;

                r.map(|_x| ())
            }
        };

        match r {
            Ok(_) => Ok(true),
            Err(err) => match err {
                DbErr::Query(err) if err.contains(postgresql::DUPLICATE_KEY_VALUE) => Ok(false),
                err => Err(err.into()),
            },
        }
    }

    async fn remove(&self, like: Like) -> crate::Result<bool> {
        let r = match like.kind() {
            LikeKind::Book => {
                like::book::Entity::delete::<like::book::ActiveModel>(like.into())
                    .exec(self.database.postgresql())
                    .await
            }

            LikeKind::BookTag => {
                like::book_tag::Entity::delete::<like::book_tag::ActiveModel>(like.into())
                    .exec(self.database.postgresql())
                    .await
            }
        };

        match r {
            Ok(x) => Ok(x.rows_affected > 0),
            Err(err) => Err(err.into()),
        }
    }
}
