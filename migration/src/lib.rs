use sea_orm_migration::{prelude::*, sea_orm::DatabaseConnection};

mod m20220520_110301_migrate_users;
mod m20220521_172503_migrate_histories_book;
mod m20220521_173415_migrate_likes_book;
mod m20220521_173450_migrate_likes_book_tag;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220520_110301_migrate_users::Migration),
            Box::new(m20220521_172503_migrate_histories_book::Migration),
            Box::new(m20220521_173415_migrate_likes_book::Migration),
            Box::new(m20220521_173450_migrate_likes_book_tag::Migration),
        ]
    }
}

pub async fn up(db: &DatabaseConnection) {
    Migrator::up(db, None).await.expect("migration");
}
