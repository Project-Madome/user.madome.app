mod app;
mod command;
mod config;
mod constant;
mod database;
mod entity;
mod error;
mod json;
mod model;
mod msg;
mod registry;
mod repository;
mod usecase;

pub use registry::RootRegistry;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
pub mod tests {
    pub use super::registry::tests::RootRegistry;
}
