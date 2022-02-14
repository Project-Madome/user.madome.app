use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityName, EntityTrait, IdenStatic, QueryFilter,
    QueryOrder, QuerySelect, Statement, TransactionTrait,
};
use uuid::Uuid;

use crate::{
    database::{postgresql::entity::notification, DatabaseSet},
    entity::{Notification, NotificationKind, NotificationSortBy, Sort},
    repository::r#trait::NotificationRepository,
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlNotificationRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlNotificationRepository {
    async fn start(&mut self) {
        notification::book::create_table(self.database.postgresql()).await;
        notification::book::tag::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl NotificationRepository for PostgresqlNotificationRepository {
    async fn get_many(
        &self,
        user_id: Uuid,
        _kind: Option<NotificationKind>,
        offset: usize,
        page: usize,
        sort_by: NotificationSortBy,
    ) -> crate::Result<Vec<Notification>> {
        // TODO: kind 추가되면 패턴 매칭
        /* let notificiations = match kind {
            Some(NotificationKind::Book) => {}
            _ => {}
        }; */

        let select =
            notification::book::Entity::find().find_with_related(notification::book::tag::Entity);

        let r = match sort_by {
            NotificationSortBy::CreatedAt(Sort::Desc) => {
                select.order_by_desc(notification::book::Column::CreatedAt)
            }
            NotificationSortBy::CreatedAt(Sort::Asc) => {
                select.order_by_asc(notification::book::Column::CreatedAt)
            }
        }
        .filter(notification::book::Column::UserId.eq(user_id))
        .limit(offset as u64)
        .offset((offset * (page - 1)) as u64)
        .all(self.database.postgresql())
        .await?;

        // log::debug!("{r:#?}");

        Ok(r.into_iter().map(Into::into).collect())
    }

    async fn add_many(
        &self,
        kind: NotificationKind,
        notifications: Vec<Notification>,
    ) -> crate::Result<()> {
        match kind {
            NotificationKind::Book => {
                let r = notifications
                    .into_iter()
                    .map(notification::book::ActiveModel::insert)
                    .fold(None::<(Vec<_>, Vec<_>)>, |acc, mut e| match acc {
                        Some(mut acc) => {
                            acc.0.push(e.0);
                            acc.1.append(&mut e.1);
                            Some(acc)
                        }
                        None => Some((vec![e.0], e.1)),
                    });

                if let Some((notifications_book, notifications_book_tag)) = r {
                    self.database
                        .postgresql()
                        .transaction::<_, (), DbErr>(|txn| {
                            Box::pin(async move {
                                /* 
                                
                                    INSERT INTO notifications_book_tag
                                
                                */
                                let (values_query, _) = notifications_book.iter().fold(
                                    (Vec::new(), 1),
                                    |(mut values, count), _| {
                                        values.push(format!(
                                            "(${}, ${}, ${}, ${})",
                                            count,
                                            count + 1,
                                            count + 2,
                                            count + 3
                                        ));
                                        (values, count + 4)
                                    },
                                );

                                let values = notifications_book
                                    .into_iter()
                                    .map(|x| {
                                        vec![
                                            x.id.unwrap().into(),
                                            x.book_id.unwrap().into(),
                                            x.user_id.unwrap().into(),
                                            x.created_at.unwrap().into(),
                                        ]
                                    })
                                    .flatten();

                                let query = format!(
                                    "INSERT INTO {} ({}, {}, {}, {}) VALUES {} ON CONFLICT (id) DO NOTHING",
                                    notification::book::Entity.table_name(),
                                    notification::book::Column::Id.as_str(),
                                    notification::book::Column::BookId.as_str(),
                                    notification::book::Column::UserId.as_str(),
                                    notification::book::Column::CreatedAt.as_str(),
                                    values_query.join(",\n")
                                );

                                let psql = txn.get_database_backend();

                                txn.execute(Statement::from_sql_and_values(psql, &query, values)).await?;

                                /* 
                                
                                    INSERT INTO notifications_book_tag
                                
                                */
                                let (values_query, _) = notifications_book_tag.iter().fold(
                                    (Vec::new(), 1),
                                    |(mut values, count), _| {
                                        values.push(format!(
                                            "(${}, ${}, ${}, ${})",
                                            count,
                                            count + 1,
                                            count + 2,
                                            count + 3
                                        ));
                                        (values, count + 4)
                                    },
                                );

                                let values = notifications_book_tag
                                    .into_iter()
                                    .map(|x| {
                                        vec![
                                            x.id.unwrap().into(),
                                            x.notification_book_id.unwrap().into(),
                                            x.tag_kind.unwrap().into(),
                                            x.tag_name.unwrap().into(),
                                        ]
                                    })
                                    .flatten();

                                let query = format!(
                                    "INSERT INTO {} ({}, {}, {}, {}) VALUES {} ON CONFLICT (id) DO NOTHING",
                                    notification::book::tag::Entity.table_name(),
                                    notification::book::tag::Column::Id.as_str(),
                                    notification::book::tag::Column::NotificationBookId.as_str(),
                                    notification::book::tag::Column::TagKind.as_str(),
                                    notification::book::tag::Column::TagName.as_str(),
                                    values_query.join(",\n")
                                );

                                let psql = txn.get_database_backend();

                                txn.execute(Statement::from_sql_and_values(psql, &query, values)).await?;

                                // TODO: if supported `ON CONFLICT DO NOTHING` to sea_orm, update to this code
                                // REF: https://github.com/SeaQL/sea-orm/issues/187
                                /* notification::book::Entity::insert_many(notifications_book)
                                    .exec(txn)
                                    .await?;

                                notification::book::tag::Entity::insert_many(
                                    notifications_book_tag,
                                )
                                .exec(txn)
                                .await?; */

                                Ok(())
                            })
                        })
                        .await?;
                }

                Ok(())
            }
        }

        /*  match noti.kind() {
            NotificationKind::Book => {
                let (notification_book, notification_book_tags) =
                    notification::book::ActiveModel::insert(noti);

                let r = self
                    .database
                    .postgresql()
                    .transaction::<_, (), DbErr>(|txn| {
                        Box::pin(async move {
                            notification::book::Entity::insert(notification_book)
                                .exec(txn)
                                .await?;

                            notification::book::tag::Entity::insert_many(notification_book_tags)
                                .exec(txn)
                                .await?;

                            Ok(())
                        })
                    })
                    .await;

                /* match r {
                    Ok(_) => Ok(true),
                    Err(TransactionError::Connection(DbErr::Query(err)))
                        if err.contains(postgresql::DUPLICATE_KEY_VALUE) =>
                    {
                        Ok(false)
                    }
                    Err(TransactionError::Connection(err)) => Err(err.into()),
                    Err(TransactionError::Transaction(err)) => Err(err.into()),
                } */
            }
            _ => {
                unreachable!()
            }
        } */
    }
}
