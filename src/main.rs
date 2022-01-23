use madome_user::RootRegistry;
use sai::System;
use simple_logger::SimpleLogger;
use tokio::signal;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    SimpleLogger::new().init().unwrap();

    let mut system = System::<RootRegistry>::new();

    system.start().await;

    signal::ctrl_c().await.unwrap();

    // system.stop().await;
}
