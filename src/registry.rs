pub use self::root_registry::RootRegistry;

mod root_registry {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        app::{HttpServer, Resolver},
        command::{
            has_book::HasBook, has_book_tag::HasBookTag, send_notification::SendNotification,
            CommandSet,
        },
        config::Config,
        database::DatabaseSet,
        repository::{
            PostgresqlDislikeRepository, PostgresqlFcmTokenRepository, PostgresqlHistoryRepository,
            PostgresqlLikeRepository, PostgresqlNotificationRepository, PostgresqlUserRepository,
            RepositorySet,
        },
    };

    combine_component_registry!(
        RootRegistry,
        [
            ServerRegistry,
            ControllerRegistry,
            RepositoryRegistry,
            CommandRegistry,
            ConfigRegistry
        ]
    );

    component_registry!(ServerRegistry, [HttpServer]);

    component_registry!(ControllerRegistry, [Resolver]);

    component_registry!(
        RepositoryRegistry,
        [
            DatabaseSet,
            RepositorySet,
            PostgresqlUserRepository,
            PostgresqlLikeRepository,
            PostgresqlDislikeRepository,
            PostgresqlNotificationRepository,
            PostgresqlFcmTokenRepository,
            PostgresqlHistoryRepository
        ]
    );

    component_registry!(
        CommandRegistry,
        [CommandSet, SendNotification, HasBook, HasBookTag]
    );

    component_registry!(ConfigRegistry, [Config]);
}
/*
#[cfg(test)]
pub mod tests {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        app::{HttpServer, Resolver},
        config::Config,
        repository::{InMemoryLikeRepository, InMemoryUserRepository, RepositorySet},
    };

    combine_component_registry!(
        RootRegistry,
        [
            ServerRegistry,
            ControllerRegistry,
            RepositoryRegistry,
            // CommandRegistry,
            ConfigRegistry
        ]
    );

    component_registry!(ServerRegistry, [HttpServer]);

    component_registry!(ControllerRegistry, [Resolver]);

    component_registry!(
        RepositoryRegistry,
        [
            RepositorySet,
            InMemoryUserRepository,
            InMemoryLikeRepository
        ]
    );

    // component_registry!(CommandRegistry, [CommandSet]);

    component_registry!(ConfigRegistry, [Config]);
}
 */
