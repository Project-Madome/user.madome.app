use sea_orm::{prelude::*, ConnectionTrait, DbBackend, Schema};

use crate::database::postgresql::entity;
use crate::entity::History;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "histories_book")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(index)]
    pub book_id: i32,
    pub user_id: Uuid,
    pub page: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
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

impl From<Model> for History {
    fn from(
        Model {
            book_id,
            page,
            user_id,
            created_at,
            updated_at,
            ..
        }: Model,
    ) -> Self {
        Self::Book {
            book_id: book_id as u32,
            page: page as usize,
            user_id,
            created_at,
            updated_at,
        }
    }
}

impl From<History> for ActiveModel {
    fn from(history: History) -> Self {
        use sea_orm::ActiveValue::*;

        match history {
            History::Book {
                book_id,
                page,
                user_id,
                created_at,
                updated_at,
            } => {
                let id = {
                    let x = format!("{book_id}{user_id}");
                    Uuid::new_v5(&Uuid::NAMESPACE_OID, x.as_bytes())
                };

                Self {
                    id: Set(id),
                    book_id: Set(book_id as i32),
                    page: Set(page as i32),
                    user_id: Set(user_id),
                    created_at: Set(created_at),
                    updated_at: Set(updated_at),
                }
            } // _ => unreachable!(), // TODO: add message to panic!
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
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
    .col(
        ColumnDef::new(Column::UpdatedAt)
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

    let schema = Schema::new(DbBackend::Postgres);

    let stmt = schema
        .create_table_from_entity(Entity)
        .if_not_exists()
        .to_owned();

    let builder = db.get_database_backend();
    db.execute(builder.build(&stmt))
        .await
        .expect("create entity::history::book table");
}
