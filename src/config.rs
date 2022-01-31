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

#[derive(Debug, Component)]
#[lifecycle]
pub struct Config {
    port: Option<u16>,

    postgres_url: Option<String>,

    madome_auth_url: Option<String>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for Config {
    async fn start(&mut self) {
        dotenv::dotenv().ok();

        self.port.replace(env("PORT"));

        self.postgres_url.replace(env("POSTGRES_URL"));

        self.madome_auth_url.replace(env("MADOME_AUTH_URL"));

        log::info!("{:?}", self);
    }
}

impl Config {
    pub fn port(&self) -> u16 {
        self.port.unwrap()
    }

    pub fn postgres_url(&self) -> &str {
        self.postgres_url.as_ref().unwrap()
    }

    pub fn madome_auth_url(&self) -> &str {
        self.madome_auth_url.as_ref().unwrap()
    }
}
