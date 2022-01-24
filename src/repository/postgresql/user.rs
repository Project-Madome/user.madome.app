use std::str::FromStr;

use sai::{Component, ComponentLifecycle, Injected};
use sea_orm::prelude::*;

use crate::{
    constant::postgresql,
    database::{postgresql::entity::user, DatabaseSet},
    entity::User,
    repository::r#trait::UserRepository,
};

#[derive(Component)]
#[lifecycle]
pub struct PostgresqlUserRepository {
    #[injected]
    database: Injected<DatabaseSet>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for PostgresqlUserRepository {
    async fn start(&mut self) {
        user::create_table(self.database.postgresql()).await;
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresqlUserRepository {
    async fn get(&self, id_or_email: String) -> crate::Result<Option<User>> {
        let maybe_id = Uuid::from_str(&id_or_email).ok();

        let user = user::Entity::find()
            .filter(
                match maybe_id {
                    Some(id) => user::Column::Id.eq(id),
                    None => user::Column::Email.eq(id_or_email),
                }, /* Condition::any()
                   .add(user::Column::Id.eq(id_or_email.clone()))
                   .add(user::Column::Email.eq(id_or_email)), */
            )
            .one(self.database.postgresql())
            .await?;

        Ok(user.map(Into::into))
    }

    async fn add(&self, user: User) -> crate::Result<Option<User>> {
        let r = user::Entity::insert::<user::ActiveModel>(user.clone().into())
            .exec(self.database.postgresql())
            .await;

        match r {
            Ok(_) => Ok(Some(user)),
            Err(err) => match err {
                DbErr::Query(err) if err.contains(postgresql::DUPLICATE_KEY_VALUE) => Ok(None),
                err => Err(err.into()),
            },
        }
    }
}
