use std::{env, fmt::Debug, str::FromStr};

use sai::{Component, ComponentLifecycle};

fn env<T>(key: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    let var = env::var(key).expect("Please set dotenv");

    var.parse().expect("Please set dotenv to valid value")
}

#[derive(Component)]
#[lifecycle]
pub struct Config {
    port: Option<u16>,

    postgres_url: Option<String>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for Config {
    async fn start(&mut self) {
        dotenv::dotenv().ok();

        self.port.replace(env("PORT"));

        self.postgres_url.replace(env("POSTGRES_URL"));
    }
}

impl Config {
    pub fn port(&self) -> u16 {
        self.port.unwrap()
    }

    pub fn postgres_url(&self) -> &str {
        self.postgres_url.as_ref().unwrap()
    }
}
