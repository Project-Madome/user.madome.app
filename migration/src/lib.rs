use sea_orm_migration::{prelude::*, sea_orm::DatabaseConnection};

mod m20220520_110301_migrate_reader_to_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220520_110301_migrate_reader_to_user::Migration)]
    }
}

pub async fn up(db: &DatabaseConnection) {
    Migrator::up(db, None).await.expect("migration");
}
