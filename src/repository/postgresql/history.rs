use itertools::Itertools;
use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, IdenStatic, PaginatorTrait, QueryFilter,
    QueryOrder, Statement,
};
use util::sea_orm::OrderByRandom;
use uuid::Uuid;

use crate::{
    database::{postgresql::entity::history, DatabaseSet},
    entity::{History, HistoryKind, HistorySortBy, Sort},
    repository::r#trait::{HistoryBy, HistoryRepository},
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlHistoryRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlHistoryRepository {
    async fn start(&mut self) {
        history::book::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl HistoryRepository for PostgresqlHistoryRepository {
    async fn get_many(
        &self,
        user_id: Uuid,
        kind: Option<HistoryKind>,
        per_page: usize,
        page: usize,
        sort_by: HistorySortBy,
    ) -> crate::Result<Vec<History>> {
        let histories = match kind {
            Some(_) => todo!(),
            // TODO: if added other kind, fixme
            None => {
                let select = history::book::Entity::find();
                let r = match sort_by {
                    HistorySortBy::CreatedAt(Sort::Desc) => {
                        select.order_by_desc(history::book::Column::CreatedAt)
                    }
                    HistorySortBy::CreatedAt(Sort::Asc) => {
                        select.order_by_asc(history::book::Column::CreatedAt)
                    }
                    HistorySortBy::UpdatedAt(Sort::Desc) => {
                        select.order_by_desc(history::book::Column::UpdatedAt)
                    }
                    HistorySortBy::UpdatedAt(Sort::Asc) => {
                        select.order_by_asc(history::book::Column::UpdatedAt)
                    }
                    HistorySortBy::Random => select.order_by_random(),
                }
                .filter(history::book::Column::UserId.eq(user_id))
                .paginate(self.database.postgresql(), per_page)
                .fetch_page(page - 1)
                .await?;

                r.into_iter().map(Into::into).collect()
            }
        };

        Ok(histories)
    }

    async fn add_or_update(&self, history: History) -> crate::Result<()> {
        match history.kind() {
            HistoryKind::Book => {
                let query = format!(
                    r#"
                    INSERT INTO
                        {table_name}(id, book_id, user_id, created_at, updated_at)
                    VALUES
                        ($1, $2, $3, $4, $4)
                    ON CONFLICT (id)
                        DO UPDATE
                            SET updated_at = $4
                "#,
                    table_name = history::book::Entity.as_str()
                );

                let db = self.database.postgresql();

                let psql = db.get_database_backend();

                let history::book::ActiveModel {
                    id: history_id,
                    book_id,
                    user_id,
                    created_at: now,
                    ..
                } = history.into();

                let _r = db
                    .execute(Statement::from_sql_and_values(
                        psql,
                        &query,
                        [
                            history_id.into_value().unwrap(),
                            book_id.into_value().unwrap(),
                            user_id.into_value().unwrap(),
                            now.into_value().unwrap(),
                        ],
                    ))
                    .await?;

                Ok(())
            }
        }
    }

    async fn get_many_by(&self, user_id: Uuid, by: HistoryBy) -> crate::Result<Vec<History>> {
        // TODO: 나중에 kind가 생기면 그때 추가하면 됨
        match by {
            HistoryBy::Book { ids } => {
                let cond = ids.into_iter().fold(Condition::any(), |acc, e| {
                    acc.add(history::book::Column::BookId.eq(e))
                });

                let histories = history::book::Entity::find()
                    .filter(history::book::Column::UserId.eq(user_id))
                    .filter(cond)
                    .all(self.database.postgresql())
                    .await?;

                Ok(histories.into_iter().map_into().collect())
            }
        }
    }

    /* async fn update(&self, history: History) -> crate::Result<Option<History>> {
        let r = history::book::Entity::update::<history::book::ActiveModel>(history.into())
            .exec(self.database.postgresql())
            .await;

        match r {
            Ok(x) => Ok(Some(x.into())),
            Err(DbErr::Query(err)) /* if err.contains(postgresql::) */ => {
                log::debug!("doesn't have history: err: {}", err);

                Ok(None)
            },
            Err(err) => Err(err.into()),
        }
    } */

    async fn remove(&self, history: History) -> crate::Result<bool> {
        let r = history::book::Entity::delete::<history::book::ActiveModel>(history.into())
            .exec(self.database.postgresql())
            .await?;

        Ok(r.rows_affected > 0)
    }
}
