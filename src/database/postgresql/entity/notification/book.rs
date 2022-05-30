use sea_orm::{prelude::*, ConnectionTrait, DbBackend, Schema};

use crate::entity::Notification;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notifications_book")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(index)]
    pub book_id: i32,
    pub user_id: Uuid,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "tag::Entity")]
    BookTag,
}

impl Related<tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BookTag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    fn id(book_id: u32, user_id: Uuid) -> Uuid {
        Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            format!("{book_id}{user_id}").as_bytes(),
        )
    }

    pub fn insert(noti: Notification) -> (Self, Vec<tag::ActiveModel>) {
        use sea_orm::ActiveValue::*;

        match noti {
            Notification::Book {
                book_id,
                user_id,
                book_tags,
                created_at,
            } => {
                let id = Self::id(book_id, user_id);

                let active_model = Self {
                    id: Set(id),
                    book_id: Set(book_id as i32),
                    user_id: Set(user_id),
                    created_at: Set(created_at),
                };

                let tag_active_models = book_tags
                    .into_iter()
                    .map(|(kind, name)| tag::ActiveModel::insert(id, kind, name))
                    .collect();

                (active_model, tag_active_models)
            }
        }
    }
}

impl From<(Model, Vec<tag::Model>)> for Notification {
    fn from((a, b): (Model, Vec<tag::Model>)) -> Self {
        let book_tags = b.into_iter().map(|x| (x.tag_kind, x.tag_name)).collect();

        Self::Book {
            user_id: a.user_id,
            book_id: a.book_id as u32,
            book_tags,
            created_at: a.created_at,
        }
    }
}

pub async fn create_table(db: &DatabaseConnection) {
    /* let stmt = Table::create()
    .table(Entity)
    .if_not_exists()
    .col(ColumnDef::new(Column::Id).uuid().primary_key())
    .col(ColumnDef::new(Column::BookId).integer().not_null())
    .col(ColumnDef::new(Column::UserId).uuid().not_null())
    .col(
        ColumnDef::new(Column::CreatedAt)
            .timestamp_with_time_zone()
            .not_null(),
    )
    .foreign_key(
        ForeignKey::create()
            .name(Column::UserId.as_str())
            .from(Entity, Column::UserId)
            .to(user::Entity, user::Column::Id)
            .on_delete(ForeignKeyAction::Cascade),
    )
    .to_owned(); */

    let schema = Schema::new(DbBackend::Postgres);

    let stmt = schema
        .create_table_from_entity(Entity)
        .if_not_exists()
        .to_owned();

    let psql = db.get_database_backend();
    db.execute(psql.build(&stmt))
        .await
        .expect("create entity::notification::book table");
}

pub mod tag {
    use sea_orm::{prelude::*, ConnectionTrait, DbBackend, Schema};

    use crate::database::postgresql::entity::{self, notification};

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "notifications_book_tag")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub notification_book_id: Uuid,
        pub tag_kind: String,
        pub tag_name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "entity::notification::book::Entity",
            from = "Column::NotificationBookId",
            to = "entity::notification::book::Column::Id",
            on_delete = "Cascade"
        )]
        NotificationBook,
    }

    impl Related<notification::book::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::NotificationBook.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}

    impl ActiveModel {
        fn id(notification_book_id: Uuid, tag_kind: &str, tag_name: &str) -> Uuid {
            Uuid::new_v5(
                &Uuid::NAMESPACE_OID,
                format!("{notification_book_id}{tag_kind}{tag_name}").as_bytes(),
            )
        }

        pub fn insert(notification_book_id: Uuid, tag_kind: String, tag_name: String) -> Self {
            use sea_orm::ActiveValue::*;

            Self {
                id: Set(Self::id(notification_book_id, &tag_kind, &tag_name)),
                notification_book_id: Set(notification_book_id),
                tag_kind: Set(tag_kind),
                tag_name: Set(tag_name),
            }
        }
    }

    pub async fn create_table(db: &DatabaseConnection) {
        /* let stmt = Table::create()
        .table(Entity)
        .if_not_exists()
        .col(ColumnDef::new(Column::Id).uuid().primary_key())
        .col(ColumnDef::new(Column::NotificationBookId).uuid().not_null())
        .col(ColumnDef::new(Column::TagKind).string().not_null())
        .col(ColumnDef::new(Column::TagName).string().not_null())
        .foreign_key(
            ForeignKey::create()
                .name(Column::NotificationBookId.as_str())
                .from(Entity, Column::NotificationBookId)
                .to(notification::book::Entity, notification::book::Column::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned(); */

        let schema = Schema::new(DbBackend::Postgres);

        let stmt = schema
            .create_table_from_entity(Entity)
            .if_not_exists()
            .to_owned();

        let psql = db.get_database_backend();
        db.execute(psql.build(&stmt))
            .await
            .expect("create entity::notification::book::tag table");
    }
}
