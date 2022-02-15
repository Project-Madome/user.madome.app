use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::{sea_query::Expr, ConnectionTrait, EntityTrait, QueryFilter, Statement};
use uuid::Uuid;

use crate::{
    database::{postgresql::entity::fcm_token, DatabaseSet},
    entity::fcm_token::FcmToken,
    repository::r#trait::FcmTokenRepository,
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlFcmTokenRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlFcmTokenRepository {
    async fn start(&mut self) {
        fcm_token::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl FcmTokenRepository for PostgresqlFcmTokenRepository {
    async fn add_or_update(
        &self,
        FcmToken {
            udid,
            user_id,
            fcm_token,
        }: FcmToken,
    ) -> crate::Result<()> {
        let query = r#"
            INSERT INTO
                fcm_token(udid, user_id, fcm_token, updated_at)
            VALUES
                ($1, $2, $3, now())
            ON CONFLICT (udid)
                DO UPDATE
                    SET fcm_token = $3
        "#;

        let db = self.database.postgresql();

        let psql = db.get_database_backend();

        let _r = db
            .execute(Statement::from_sql_and_values(
                psql,
                query,
                [udid.into(), user_id.into(), fcm_token.into()],
            ))
            .await?;

        Ok(())
    }

    async fn get_many(&self, user_ids: Vec<Uuid>) -> crate::Result<Vec<String>> {
        let r = fcm_token::Entity::find()
            .filter(Expr::col(fcm_token::Column::UserId).is_in(user_ids))
            .filter(
                Expr::expr(Expr::cust(r#"now() - "updated_at""#))
                    .less_than(Expr::cust("'30 days'")),
            )
            .all(self.database.postgresql())
            .await?;

        Ok(r.into_iter().map(|x| x.fcm_token).collect())
    }
}
