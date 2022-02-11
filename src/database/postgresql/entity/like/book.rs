use chrono::{FixedOffset, Utc};
use sea_orm::{
    prelude::*,
    sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Table},
    ConnectionTrait, DeriveEntityModel, DeriveRelation, EnumIter,
};
use uuid::Uuid;

use crate::database::postgresql::entity;
use crate::entity::Like;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "likes_book")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub book_id: i32,
    pub user_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "entity::user::Entity",
        from = "Column::UserId",
        to = "entity::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Like {
    fn from(
        Model {
            book_id,
            user_id,
            created_at,
            ..
        }: Model,
    ) -> Self {
        Self::Book {
            book_id: book_id as u32,
            user_id,
            created_at: created_at.with_timezone(&Utc),
        }
    }
}

impl From<Like> for ActiveModel {
    fn from(like: Like) -> Self {
        use sea_orm::ActiveValue::*;

        match like {
            Like::Book {
                book_id,
                user_id,
                created_at,
            } => {
                let id = {
                    let x = format!("{book_id}{user_id}");
                    Uuid::new_v5(&Uuid::NAMESPACE_OID, x.as_bytes())
                };

                let utc = FixedOffset::east(0);

                Self {
                    id: Set(id),
                    book_id: Set(book_id as i32),
                    user_id: Set(user_id),
                    created_at: Set(created_at.with_timezone(&utc)),
                }
            }
            _ => unreachable!(),
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
    let smtm = Table::create()
        .table(Entity)
        .if_not_exists()
        .col(ColumnDef::new(Column::Id).uuid().primary_key())
        .col(ColumnDef::new(Column::BookId).integer().not_null())
        .col(ColumnDef::new(Column::UserId).uuid().not_null())
        .col(
            ColumnDef::new(Column::CreatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("user_id")
                .from(Entity, Column::UserId)
                .to(entity::user::Entity, entity::user::Column::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned();

    let builder = db.get_database_backend();
    db.execute(builder.build(&smtm))
        .await
        .expect("create entity::like::book table");
}
