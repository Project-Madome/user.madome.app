use log::Level;
use madome_user::RootRegistry;
use sai::System;
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut system = System::<RootRegistry>::new();

    system.start().await;

    signal::ctrl_c().await.unwrap();

    // system.stop().await;
}
