use log::Level;
use madome_user::{release, RootRegistry};
use sai::System;
use tokio::signal::{self, unix::SignalKind};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let log_level = if release() { Level::Info } else { Level::Debug };

    simple_logger::init_with_level(log_level).unwrap();

    let mut system = System::<RootRegistry>::new();

    system.start().await;

    let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sigterm.recv() => {},
        _ = async { signal::ctrl_c().await.expect("failed to listen for ctrl_c event") } => {}
    };

    system.stop().await;

    log::info!("gracefully shutdown the app");
}
// Check TypeId
/*
    fn types() -> Vec<(String, TypeId)> {
    vec![
        (
            "HttpServer".to_string(),
            TypeId::of::<Injected<HttpServer>>(),
        ),
        ("Resolver".to_string(), TypeId::of::<Injected<Resolver>>()),
        (
            "RepositorySet".to_string(),
            TypeId::of::<Injected<RepositorySet>>(),
        ),
        (
            "DatabaseSet".to_string(),
            TypeId::of::<Injected<DatabaseSet>>(),
        ),
        (
            "PostgresqlUserRepository".to_string(),
            TypeId::of::<Injected<PostgresqlUserRepository>>(),
        ),
        (
            "PostgresqlLikeRepository".to_string(),
            TypeId::of::<Injected<PostgresqlLikeRepository>>(),
        ),
        (
            "HttpConfig".to_string(),
            TypeId::of::<Injected<HttpConfig>>(),
        ),
        (
            "PostgresqlConfig".to_string(),
            TypeId::of::<Injected<PostgresqlConfig>>(),
        ),
        (
            "MadomeConfig".to_string(),
            TypeId::of::<Injected<MadomeConfig>>(),
        ),
    ]
}

    for tid in types() {
        println!("{tid:?}");
    } */
