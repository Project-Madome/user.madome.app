use hyper::header;
use sai::{Component, Injected};

use crate::{command::r#trait::Command, config::Config, error::CommandError};

#[derive(Component)]
pub struct HasBook {
    #[injected]
    config: Injected<Config>,
}

impl HasBook {
    pub async fn has_book(&self, book_id: u32) -> Result<bool, Error> {
        let url = format!("{}/command", self.config.library_url());

        log::debug!("{url}");

        #[derive(serde::Serialize)]
        struct Req<'a> {
            pub kind: &'a str,
            pub book_id: u32,
        }

        let body = Req {
            kind: "has_book",
            book_id,
        };

        let resp = reqwest::Client::new()
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .await?;

        #[derive(serde::Deserialize)]
        struct Resp {
            pub has: bool,
        }

        let Resp { has } = resp.json::<Resp>().await?;

        Ok(has)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        CommandError::from(err).into()
    }
}

#[async_trait::async_trait]
impl Command<u32, bool> for HasBook {
    type Error = crate::Error;

    async fn execute(&self, book_id: u32) -> Result<bool, Self::Error> {
        let x = self.has_book(book_id).await?;

        Ok(x)
    }
}

pub mod r#trait {
    use crate::command::r#trait::Command;

    pub trait HasBook: Command<(), ()> {}
}
