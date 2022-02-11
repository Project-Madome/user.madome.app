use chrono::{FixedOffset, Utc};
use sea_orm::sea_query::{ForeignKey, ForeignKeyAction, Table};
use sea_orm::ConnectionTrait;
use sea_orm::{prelude::*, sea_query::ColumnDef, DeriveEntityModel, DeriveRelation, EnumIter};
use uuid::Uuid;

use crate::database::postgresql::entity;
use crate::entity::Like;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "likes_book_tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tag_kind: String,
    pub tag_name: String,
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
            tag_kind,
            tag_name,
            user_id,
            created_at,
            ..
        }: Model,
    ) -> Self {
        Self::BookTag {
            tag_kind,
            tag_name,
            user_id,
            created_at: created_at.with_timezone(&Utc),
        }
    }
}

impl From<Like> for ActiveModel {
    fn from(like: Like) -> Self {
        use sea_orm::ActiveValue::*;

        match like {
            Like::BookTag {
                tag_kind,
                tag_name,
                user_id,
                created_at,
            } => {
                let id = {
                    let x = format!("{tag_kind}{tag_name}{user_id}");
                    Uuid::new_v5(&Uuid::NAMESPACE_OID, x.as_bytes())
                };

                let utc = FixedOffset::east(0);

                Self {
                    id: Set(id),
                    tag_kind: Set(tag_kind),
                    tag_name: Set(tag_name),
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
        .col(ColumnDef::new(Column::TagKind).string().not_null())
        .col(ColumnDef::new(Column::TagName).string().not_null())
        .col(ColumnDef::new(Column::UserId).uuid().not_null())
        .col(
            ColumnDef::new(Column::CreatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("user_id")
                .from(entity::like::book_tag::Entity, Column::UserId)
                .to(entity::user::Entity, entity::user::Column::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned();

    let builder = db.get_database_backend();
    db.execute(builder.build(&smtm))
        .await
        .expect("create entity::like::book_tag table");
}
