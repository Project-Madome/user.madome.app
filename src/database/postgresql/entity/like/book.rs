use sea_orm::{
    prelude::*, ConnectionTrait, DbBackend, DeriveEntityModel, DeriveRelation, EnumIter, Schema,
};
use uuid::Uuid;

use crate::database::postgresql::entity;
use crate::entity::{Dislike, Like};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "likes_book")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(index)]
    pub book_id: i32,
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
            book_id,
            user_id,
            created_at,
            ..
        } = self;

        Like::Book {
            book_id: book_id as u32,
            user_id,
            created_at,
        }
    }

    pub fn into_dislike(self) -> Dislike {
        let Model {
            book_id,
            user_id,
            created_at,
            ..
        } = self;

        Dislike::Book {
            book_id: book_id as u32,
            user_id,
            created_at,
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

                /* let r =  */
                Self {
                    id: Set(id),
                    book_id: Set(book_id as i32),
                    user_id: Set(user_id),
                    is_dislike: Set(false),
                    created_at: Set(created_at),
                }

                // log::debug!("like::book::{r:?}");

                // r
            }
            _ => unreachable!(), // TODO: add message to panic!
        }
    }
}

impl From<Dislike> for ActiveModel {
    fn from(dislike: Dislike) -> Self {
        use sea_orm::ActiveValue::*;

        match dislike {
            Dislike::Book {
                book_id,
                user_id,
                created_at,
            } => {
                let id = {
                    let x = format!("{book_id}{user_id}");
                    Uuid::new_v5(&Uuid::NAMESPACE_OID, x.as_bytes())
                };

                /* let r =  */
                Self {
                    id: Set(id),
                    book_id: Set(book_id as i32),
                    user_id: Set(user_id),
                    is_dislike: Set(true),
                    created_at: Set(created_at),
                }

                // log::debug!("like::book::{r:?}");

                // r
            }
            _ => unreachable!(), // TODO: add message to panic!
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
    .col(ColumnDef::new(Column::BookId).integer().not_null())
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
        .expect("create entity::like::book table");
}
