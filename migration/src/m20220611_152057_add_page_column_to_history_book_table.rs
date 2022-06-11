use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220611_152057_add_page_column_to_history_book_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let stmt = Table::alter()
            .table(Alias::new("histories_book"))
            .add_column(
                ColumnDef::new(Alias::new("page"))
                    .integer()
                    .not_null()
                    .default::<i32>(1),
            )
            .to_owned();

        manager.alter_table(stmt).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
