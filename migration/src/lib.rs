use sea_orm_migration::{prelude::*, sea_orm::DatabaseConnection};

mod m20220520_110301_migrate_users;
mod m20220521_172503_migrate_histories_book;
mod m20220521_173415_migrate_likes_book;
mod m20220521_173450_migrate_likes_book_tag;
mod m20220611_152057_add_page_column_to_history_book_table;
mod m20220611_161637_add_is_dislike_column_to_likes_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220520_110301_migrate_users::Migration),
            Box::new(m20220521_172503_migrate_histories_book::Migration),
            Box::new(m20220521_173415_migrate_likes_book::Migration),
            Box::new(m20220521_173450_migrate_likes_book_tag::Migration),
            // Box::new(m20220611_152057_add_page_column_to_history_book_table::Migration),
            // Box::new(m20220611_161637_add_is_dislike_column_to_likes_table::Migration),
        ]
    }
}

pub async fn up(db: &DatabaseConnection) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
