use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, IdenStatic, PaginatorTrait,
    QueryFilter, QueryOrder, Statement, TryGetable,
};
use util::sea_orm::OrderByRandom;
use uuid::Uuid;

use crate::{
    constant::postgresql,
    database::{postgresql::entity::like, DatabaseSet},
    entity::{Dislike, DislikeKind, DislikeSortBy, Sort},
    repository::r#trait::DislikeRepository,
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlDislikeRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlDislikeRepository {
    async fn start(&mut self) {

        // like::book::create_table(self.database.postgresql()).await;
        // like::book_tag::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl DislikeRepository for PostgresqlDislikeRepository {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<DislikeKind>,
        per_page: usize,
        page: usize,
        sort_by: DislikeSortBy,
    ) -> crate::Result<Vec<Dislike>> {
        let dislikes = match kind {
            Some(DislikeKind::Book) => {
                let select = like::book::Entity::find();
                let r = match sort_by {
                    DislikeSortBy::Random => select.order_by_random(),
                    DislikeSortBy::CreatedAt(Sort::Desc) => {
                        select.order_by_desc(like::book::Column::CreatedAt)
                    }
                    DislikeSortBy::CreatedAt(Sort::Asc) => {
                        select.order_by_asc(like::book::Column::CreatedAt)
                    }
                }
                .filter(
                    like::book::Column::UserId
                        .eq(user_id)
                        .and(like::book::Column::IsDislike.eq(true)),
                )
                /* .query()
                .per_page(per_page * (page - 1))
                .limit(per_page) */
                .paginate(self.database.postgresql(), per_page) // case1
                .fetch_page(page - 1) // case2
                .await?;

                r.into_iter().map(like::book::Model::into_dislike).collect()
            }

            Some(DislikeKind::BookTag) => {
                let select = like::book_tag::Entity::find();
                let r = match sort_by {
                    DislikeSortBy::Random => select.order_by_random(),
                    DislikeSortBy::CreatedAt(Sort::Desc) => {
                        select.order_by_desc(like::book_tag::Column::CreatedAt)
                    }
                    DislikeSortBy::CreatedAt(Sort::Asc) => {
                        select.order_by_asc(like::book_tag::Column::CreatedAt)
                    }
                }
                .order_by_desc(like::book_tag::Column::CreatedAt)
                .filter(
                    like::book_tag::Column::UserId
                        .eq(user_id)
                        .and(like::book_tag::Column::IsDislike.eq(true)),
                )
                .paginate(self.database.postgresql(), per_page)
                .fetch_page(page - 1)
                .await?;

                r.into_iter()
                    .map(like::book_tag::Model::into_dislike)
                    .collect()
            }

            None => {
                let sort_by = match sort_by {
                    DislikeSortBy::CreatedAt(Sort::Desc) => "created_at DESC",
                    DislikeSortBy::CreatedAt(Sort::Asc) => "created_at ASC",
                    DislikeSortBy::Random => "RANDOM() DESC",
                };

                let query = format!(
                    r#"
                    SELECT * FROM
                    (
                        SELECT id, user_id, book_id, NULL AS tag_kind, NULL AS tag_name, created_at
                            FROM {like_book_table}
                            WHERE user_id = $1 AND is_dislike = true
                        UNION ALL
                        SELECT id, user_id, NULL, tag_kind, tag_name, created_at
                            FROM {like_book_tag_table}
                            WHERE user_id = $1 AND is_dislike = true
                    ) AS a
                    ORDER BY {sort_by}
                    LIMIT $2
                    OFFSET $3
                    "#,
                    like_book_table = like::book::Entity.as_str(),
                    like_book_tag_table = like::book_tag::Entity.as_str(),
                );

                let db = self.database.postgresql();
                let psql = db.get_database_backend();

                let query_results = db
                    .query_all(Statement::from_sql_and_values(
                        psql,
                        &query,
                        [
                            user_id.into(),
                            (per_page as i64).into(),
                            ((per_page * (page - 1)) as i64).into(),
                        ],
                    ))
                    .await?;

                query_results
                    .iter()
                    .map(|x| {
                        // partitioning by DislikeKind
                        let like_kind = {
                            // TODO: if another kind added, change kind partitioning logic
                            let r =
                                Option::<i32>::try_get(x, "", like::book::Column::BookId.as_str())
                                    .map_err(DbErr::from)?;

                            log::debug!("{r:?}");

                            match r {
                                Some(_) => DislikeKind::Book,
                                None => DislikeKind::BookTag,
                            }
                        };

                        // into Dislike
                        match like_kind {
                            DislikeKind::Book => {
                                let model = like::book::Model::from_query_result(x, "")?;

                                Ok(model.into_dislike())
                            }
                            DislikeKind::BookTag => {
                                let model = like::book_tag::Model::from_query_result(x, "")?;

                                Ok(model.into_dislike())
                            }
                        }
                    })
                    .collect::<Result<_, DbErr>>()?
            }
        };

        Ok(dislikes)
    }

    /* async fn filter(&self, user_id: Uuid, dislikes: Vec<Dislike>) -> crate::Result<bool> {
        let dislikes_book = dislikes.iter().filter(|x| x.kind() == DislikeKind::Book);
        let dislikes_book_tag = dislikes.iter().filter(|x| x.kind() == DislikeKind::BookTag);

        todo!()
    } */

    async fn add(&self, dislike: Dislike) -> crate::Result<bool> {
        let r = match dislike.kind() {
            DislikeKind::Book => {
                let r = like::book::Entity::insert::<like::book::ActiveModel>(dislike.into())
                    .exec(self.database.postgresql())
                    .await;

                r.map(|_x| ())
            }

            DislikeKind::BookTag => {
                let r =
                    like::book_tag::Entity::insert::<like::book_tag::ActiveModel>(dislike.into())
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

    async fn remove(&self, dislike: Dislike) -> crate::Result<bool> {
        let r = match dislike.kind() {
            DislikeKind::Book => {
                like::book::Entity::delete::<like::book::ActiveModel>(dislike.into())
                    .exec(self.database.postgresql())
                    .await
            }

            DislikeKind::BookTag => {
                like::book_tag::Entity::delete::<like::book_tag::ActiveModel>(dislike.into())
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
