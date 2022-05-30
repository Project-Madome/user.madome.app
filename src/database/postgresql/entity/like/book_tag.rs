use sea_orm::{prelude::*, DeriveEntityModel, DeriveRelation, EnumIter};
use sea_orm::{ConnectionTrait, DbBackend, Schema};
use uuid::Uuid;

use crate::database::postgresql::entity;
use crate::entity::{Dislike, Like};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "likes_book_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(index)]
    pub tag_kind: String,
    #[sea_orm(index)]
    pub tag_name: String,
    pub user_id: Uuid,
    pub is_dislike: bool,
    pub created_at: DateTimeUtc,
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

impl Model {
    pub fn into_like(self) -> Like {
        let Model {
            tag_kind,
            tag_name,
            user_id,
            created_at,
            ..
        } = self;

        Like::BookTag {
            tag_kind,
            tag_name,
            user_id,
            created_at,
        }
    }

    pub fn into_dislike(self) -> Dislike {
        let Model {
            tag_kind,
            tag_name,
            user_id,
            created_at,
            ..
        } = self;

        Dislike::BookTag {
            tag_kind,
            tag_name,
            user_id,
            created_at,
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

                Self {
                    id: Set(id),
                    tag_kind: Set(tag_kind),
                    tag_name: Set(tag_name),
                    user_id: Set(user_id),
                    is_dislike: Set(false),
                    created_at: Set(created_at),
                }
            }
            _ => unreachable!(),
        }
    }
}

impl From<Dislike> for ActiveModel {
    fn from(like: Dislike) -> Self {
        use sea_orm::ActiveValue::*;

        match like {
            Dislike::BookTag {
                tag_kind,
                tag_name,
                user_id,
                created_at,
            } => {
                let id = {
                    let x = format!("{tag_kind}{tag_name}{user_id}");
                    Uuid::new_v5(&Uuid::NAMESPACE_OID, x.as_bytes())
                };

                Self {
                    id: Set(id),
                    tag_kind: Set(tag_kind),
                    tag_name: Set(tag_name),
                    user_id: Set(user_id),
                    is_dislike: Set(true),
                    created_at: Set(created_at),
                }
            }
            _ => unreachable!(),
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
    let schema = Schema::new(DbBackend::Postgres);

    let stmt = schema
        .create_table_from_entity(Entity)
        .if_not_exists()
        .to_owned();

    /* let stmt = Table::create()
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
            .name(Column::UserId.as_str())
            .from(Entity, Column::UserId)
            .to(entity::user::Entity, entity::user::Column::Id)
            .on_delete(ForeignKeyAction::Cascade),
    )
    .to_owned(); */

    let builder = db.get_database_backend();
    db.execute(builder.build(&stmt))
        .await
        .expect("create entity::like::book_tag table");
}
