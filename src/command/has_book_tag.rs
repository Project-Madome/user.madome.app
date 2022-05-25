use hyper::header;
use sai::{Component, Injected};

use crate::{command::r#trait::Command, config::Config, error::CommandError};

#[derive(Component)]
pub struct HasBookTag {
    #[injected]
    config: Injected<Config>,
}

impl HasBookTag {
    pub async fn has_book_tag(&self, tag_kind: &str, tag_name: &str) -> Result<bool, Error> {
        let url = format!("{}/command", self.config.library_url());

        #[derive(serde::Serialize)]
        struct Req<'a> {
            pub kind: &'a str,
            pub book_tag: (&'a str, &'a str),
        }

        let body = Req {
            kind: "has_book_tag",
            book_tag: (tag_kind, tag_name),
        };

        let resp = reqwest::Client::new()
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&body).unwrap())
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
impl Command<(String, String), bool> for HasBookTag {
    type Error = crate::Error;

    async fn execute(&self, (tag_kind, tag_name): (String, String)) -> Result<bool, Self::Error> {
        let x = self.has_book_tag(&tag_kind, &tag_name).await?;

        Ok(x)
    }
}

pub mod r#trait {
    use crate::command::r#trait::Command;

    pub trait HasBookTag: Command<(), ()> {}
}
