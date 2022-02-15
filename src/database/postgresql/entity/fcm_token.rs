use chrono::Utc;
use sea_orm::{
    prelude::*,
    sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Table},
    ConnectionTrait,
};

use crate::entity::fcm_token::FcmToken;

use super::user;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "fcm_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub udid: Uuid,
    pub user_id: Uuid,
    pub fcm_token: String,
    pub updated_at: DateTimeUtc,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<FcmToken> for ActiveModel {
    fn from(
        FcmToken {
            udid,
            user_id,
            fcm_token,
        }: FcmToken,
    ) -> Self {
        use sea_orm::ActiveValue::*;

        let now = Utc::now();

        Self {
            udid: Set(udid),
            user_id: Set(user_id),
            fcm_token: Set(fcm_token),
            updated_at: Set(now),
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
    let stmt = Table::create()
        .table(Entity)
        .if_not_exists()
        .col(ColumnDef::new(Column::Udid).uuid().primary_key())
        .col(ColumnDef::new(Column::UserId).uuid().not_null())
        .col(ColumnDef::new(Column::FcmToken).string().not_null())
        .col(
            ColumnDef::new(Column::UpdatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name(Column::UserId.as_str())
                .from(Entity, Column::UserId)
                .to(user::Entity, user::Column::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned();

    let psql = db.get_database_backend();

    db.execute(psql.build(&stmt))
        .await
        .expect("create table entity::fcm_token");
}
