use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220521_172503_migrate_histories_book"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        /*
        @Entity('history')
        export default class History {
            @PrimaryGeneratedColumn('uuid')
            public id!: string;

            @Index()
            @Column('int')
            public uid!: number; // book.id

            @Index()
            @Column('uuid')
            public reader_id!: string;

            @Column('varchar')
            public type!: 'book' | 'video';

            @CreateDateColumn()
            @Column('timestamptz')
            public created_at!: Date;

            @UpdateDateColumn()
            @Column('timestamptz')
            public updated_at!: Date;
        }
        */

        let migrate_histories = format!(
            r#"
            INSERT INTO {table_name}(id, book_id, user_id, created_at, updated_at)
            SELECT id, uid, reader_id, created_at, updated_at FROM history
            "#,
            table_name = "histories_book"
        );

        let conn = manager.get_connection();
        let backend = manager.get_database_backend();

        conn.execute(Statement::from_sql_and_values(
            backend,
            &migrate_histories,
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
