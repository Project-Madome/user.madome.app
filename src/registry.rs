pub use self::root_registry::RootRegistry;

mod root_registry {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        app::{HttpServer, Resolver},
        config::Config,
        database::DatabaseSet,
        repository::{PostgresqlUserRepository, RepositorySet},
    };

    combine_component_registry!(
        RootRegistry,
        [
            ServerRegistry,
            ControllerRegistry,
            RepositoryRegistry, // CommandRegistry,
            ConfigRegistry
        ]
    );

    component_registry!(ServerRegistry, [HttpServer]);

    component_registry!(ControllerRegistry, [Resolver]);

    component_registry!(
        RepositoryRegistry,
        [DatabaseSet, RepositorySet, PostgresqlUserRepository]
    );

    // component_registry!(CommandRegistry, [CommandSet]);

    component_registry!(ConfigRegistry, [Config]);
}

#[cfg(test)]
pub mod tests {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        app::{HttpServer, Resolver},
        config::Config,
        repository::{InMemoryUserRepository, RepositorySet},
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

    component_registry!(RepositoryRegistry, [RepositorySet, InMemoryUserRepository]);

    // component_registry!(CommandRegistry, [CommandSet]);

    component_registry!(ConfigRegistry, [Config]);
}
