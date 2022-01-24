use chrono::{FixedOffset, Utc};
use sea_orm::{
    entity::prelude::*,
    sea_query::{ColumnDef, Table},
    ConnectionTrait,
};

use crate::entity::User;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(column_type = "SmallInteger")]
    pub role: i16,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for User {
    fn from(
        Model {
            id,
            name,
            email,
            role,
            created_at,
            updated_at,
        }: Model,
    ) -> Self {
        Self {
            id,
            name,
            email,
            role: (role as u8).into(),
            created_at: created_at.with_timezone(&Utc),
            updated_at: updated_at.with_timezone(&Utc),
        }
    }
}

impl From<User> for ActiveModel {
    fn from(
        User {
            id,
            name,
            email,
            role,
            created_at,
            updated_at,
        }: User,
    ) -> Self {
        use sea_orm::ActiveValue::*;

        let utc = FixedOffset::east(0);

        let role: u8 = role.into();

        Self {
            id: Set(id),
            name: Set(name),
            email: Set(email),
            role: Set(role as i16),
            created_at: Set(created_at.with_timezone(&utc)),
            updated_at: Set(updated_at.with_timezone(&utc)),
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
    let smtm = Table::create()
        .table(Entity)
        .if_not_exists()
        .col(ColumnDef::new(Column::Id).uuid().primary_key())
        .col(ColumnDef::new(Column::Name).string().unique_key())
        .col(ColumnDef::new(Column::Email).string().unique_key())
        .col(ColumnDef::new(Column::Role).small_integer().not_null())
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
        .to_owned();

    let builder = db.get_database_backend();
    db.execute(builder.build(&smtm))
        .await
        .expect("create entity::User table");
}
