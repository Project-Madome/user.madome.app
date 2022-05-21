use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, FromQueryResult, Statement},
};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220521_173450_migrate_likes_book_tag"
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

        let conn = manager.get_connection();
        let backend = manager.get_database_backend();

        for kind in ["artist", "series", "character", "group"] {
            migrate_x_to_likes_book_tag(kind, conn, backend).await;
        }

        migrate_tag_to_likes_book_tag(conn, backend).await;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        todo!()
    }
}

async fn migrate_x_to_likes_book_tag(
    kind: &str,
    conn: &DatabaseConnection,
    backend: DatabaseBackend,
) {
    let query = format!(
        r#"
        INSERT INTO {table_name}(id, tag_kind, tag_name, user_id, created_at)
        SELECT id, $1, value, reader_id, created_at FROM favorite WHERE type = $1
        "#,
        table_name = "likes_book_tag"
    );

    conn.execute(Statement::from_sql_and_values(
        backend,
        &query,
        [kind.into()],
    ))
    .await
    .unwrap();
}

async fn migrate_tag_to_likes_book_tag(conn: &DatabaseConnection, backend: DatabaseBackend) {
    let select_tag_favorite =
        r#"SELECT id, value, reader_id, created_at FROM favorite WHERE type = 'tag'"#;

    let xs = conn
        .query_all(Statement::from_string(
            backend,
            select_tag_favorite.to_string(),
        ))
        .await
        .unwrap();

    let tag_favorites = xs
        .iter()
        .map(|x| tag_favorite::TagFavorite::from_query_result(x, ""))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let likes = tag_favorites
        .into_iter()
        .map(likes_book_tag::LikeBookTag::from);

    let (vars, values): (Vec<_>, Vec<Vec<Value>>) = likes
        .enumerate()
        .map(|(i, x)| ((i + 1) * 5, x))
        .map(|(i, x)| {
            (
                format!("(${}, ${}, ${}, ${}, ${})", i - 4, i - 3, i - 2, i - 1, i),
                vec![
                    x.id.into(),
                    x.tag_kind.into(),
                    x.tag_name.into(),
                    x.user_id.into(),
                    x.created_at.into(),
                ],
            )
        })
        .unzip();

    let vars = vars.join(",");
    let values = values.into_iter().flatten();

    let insert_likes_book_tag = format!(
        r#"
            INSERT INTO {table_name}(id, tag_kind, tag_name, user_id, created_at)
            VALUES {vars}
            "#,
        table_name = "likes_book_tag"
    );

    conn.execute(Statement::from_sql_and_values(
        backend,
        &insert_likes_book_tag,
        values,
    ))
    .await
    .unwrap();
}

mod likes_book_tag {
    use sea_orm_migration::sea_orm::prelude::*;

    fn to_tag(x: &str) -> Option<(&str, &str)> {
        if x.starts_with("female") || x.starts_with("male") {
            let mut y = x.splitn(2, ' ');
            let (kind, name) = (y.next()?, y.next()?);

            Some((kind, name))
        } else {
            Some(("misc", x))
        }
    }

    pub struct LikeBookTag {
        pub id: Uuid,
        pub tag_kind: String,
        pub tag_name: String,
        pub user_id: Uuid,
        pub created_at: DateTimeUtc,
    }

    impl From<super::tag_favorite::TagFavorite> for LikeBookTag {
        fn from(x: super::tag_favorite::TagFavorite) -> Self {
            let (kind, name) = to_tag(&x.value).unwrap();

            Self {
                id: x.id,
                tag_kind: kind.to_string(),
                tag_name: name.to_string(),
                user_id: x.reader_id,
                created_at: x.created_at,
            }
        }
    }
}

mod tag_favorite {
    use sea_orm_migration::sea_orm::{prelude::*, FromQueryResult};

    pub struct TagFavorite {
        pub id: Uuid,
        pub value: String,
        pub reader_id: Uuid,
        pub created_at: DateTimeUtc,
    }

    impl FromQueryResult for TagFavorite {
        fn from_query_result(x: &QueryResult, pre: &str) -> Result<Self, DbErr> {
            Ok(Self {
                id: x.try_get(pre, "id")?,
                value: x.try_get(pre, "value")?,
                reader_id: x.try_get(pre, "reader_id")?,
                created_at: x.try_get(pre, "created_at")?,
            })
        }
    }
}
