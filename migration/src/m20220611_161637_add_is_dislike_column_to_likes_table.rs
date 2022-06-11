use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220611_161637_add_is_dislike_column_to_likes_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let stmt = add_is_dislike_column_to_likes_table("likes_book");

        manager.alter_table(stmt).await?;

        let stmt = add_is_dislike_column_to_likes_table("likes_book_tag");

        manager.alter_table(stmt).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}

fn add_is_dislike_column_to_likes_table(table: &str) -> TableAlterStatement {
    Table::alter()
        .table(Alias::new(table))
        .add_column(
            ColumnDef::new(Alias::new("is_dislike"))
                .boolean()
                .not_null()
                .default::<bool>(false),
        )
        .to_owned()
}
