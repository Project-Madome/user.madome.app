use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220520_110301_migrate_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        /*
            @Entity("reader")
            export default class Reader {
                @PrimaryGeneratedColumn("uuid")
                public id!: string;

                @Column("varchar", { unique: true })
                public username!: string;

                @Column("varchar", { unique: true })
                public email!: string;

                @Column("int4", { default: Role.NORMAL })
                public role!: Role;

                @CreateDateColumn()
                @Column("timestamptz")
                public created_at!: Date;
            }
        */

        let conn = manager.get_connection();
        let backend = manager.get_database_backend();

        /*         let select_all_readers = r#"
            SELECT * FROM reader
        "#;

        let r = conn
            .query_all(Statement::from_sql_and_values(
                backend,
                select_all_readers,
                [],
            ))
            .await
            .expect("select all readers");

        let readers = r
            .iter()
            .map(|x| Reader::from_query_result(x, ""))
            .collect::<Result<Vec<_>, _>>()
            .expect("reader from query result"); */

        // user::create_table(conn).await;
        // .expect("create table of user");

        let migrate_readers_to_users = format!(
            r#"
            INSERT INTO {table_name}(id, name, email, role, created_at, updated_at)
            SELECT id, username, email, role, created_at, created_at FROM reader
            "#,
            table_name = "users" // user::Table.as_str()
        );

        conn.execute(Statement::from_sql_and_values(
            backend,
            &migrate_readers_to_users,
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

/* mod user {
    use sea_orm_migration::sea_orm::{
        sea_query::{self, ColumnDef},
        ConnectionTrait, DatabaseConnection, Iden,
    };

    pub struct Table;

    impl Table {
        pub fn as_str(&self) -> &str {
            "users"
        }
    }

    impl Iden for Table {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }

    pub enum Column {
        Id,
        Name,
        Email,
        Role,
        CreatedAt,
        UpdatedAt,
    }

    impl Column {
        pub fn as_str(&self) -> &str {
            match self {
                Column::Id => "id",
                Column::Name => "name",
                Column::Email => "email",
                Column::Role => "role",
                Column::CreatedAt => "created_at",
                Column::UpdatedAt => "updated_at",
            }
        }
    }

    impl Iden for Column {
        fn unquoted(&self, s: &mut dyn std::fmt::Write) {
            s.write_str(self.as_str()).unwrap();
        }
    }

    pub async fn create_table(db: &DatabaseConnection) {
        let smtm = sea_query::Table::create()
            .table(Table)
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).uuid().primary_key())
            .col(ColumnDef::new(Column::Name).string().unique_key())
            .col(ColumnDef::new(Column::Email).string().unique_key())
            .col(ColumnDef::new(Column::Role).small_integer().not_null())
            .col(
                ColumnDef::new(Column::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(Column::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .to_owned();

        let builder = db.get_database_backend();
        db.execute(builder.build(&smtm))
            .await
            .expect("create user table");
    }
} */
/*
struct Reader {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: i32,
    pub created_at: DateTimeUtc,
}

impl Reader {
    fn into_user_active_model(self) -> entity::user::ActiveModel {
        use sea_orm::ActiveValue::*;

        entity::user::ActiveModel {
            id: Set(self.id),
            name: Set(self.username),
            email: Set(self.email),
            role: Set(self.role as i16),
            created_at: Set(self.created_at),
            updated_at: Set(self.created_at),
        }
    }
}

impl FromQueryResult for Reader {
    fn from_query_result(res: &QueryResult, pre: &str) -> Result<Self, DbErr> {
        let id: Uuid = res.try_get(pre, "id")?;
        let username: String = res.try_get(pre, "username")?;
        let email: String = res.try_get(pre, "email")?;
        let role: i32 = res.try_get(pre, "role")?;
        let created_at: DateTimeUtc = res.try_get(pre, "created_at")?;

        Ok(Self {
            id,
            username,
            email,
            role,
            created_at,
        })
    }
}
 */
