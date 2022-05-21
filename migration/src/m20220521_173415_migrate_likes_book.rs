use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220521_173415_migrate_likes_book"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        /*
        type FavoriteType =
            | 'group'
            | 'character'
            | 'tag'
            | 'artist'
            | 'series'
            | 'book';

        @Entity('favorite')
        export default class Favorite {
            @PrimaryGeneratedColumn('uuid')
            public id!: string;

            @Column('varchar')
            public value!: string;

            @Column('varchar', { length: 30, default: '' })
            public memo!: string;

            @Index()
            @Column('varchar')
            public type!: FavoriteType;

            @Index()
            @Column('uuid')
            public reader_id!: string;

            @CreateDateColumn()
            @Column('timestamptz')
            public created_at!: Date;
        }
        */

        let migrate_likes_book = format!(
            r#"
            INSERT INTO {table_name}(id, book_id, user_id, created_at)
            SELECT id, value::int4, reader_id, created_at FROM favorite WHERE type = 'book'
            "#,
            table_name = "likes_book"
        );

        let conn = manager.get_connection();
        let backend = manager.get_database_backend();

        conn.execute(Statement::from_sql_and_values(
            backend,
            &migrate_likes_book,
            [],
        ))
        .await
        .unwrap();

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}
