use fcm_sdk::FirebaseCloudMessaging;
use sai::{Component, ComponentLifecycle};

use crate::command::r#trait::Command;

#[derive(Debug)]
pub struct Message {
    pub title: String,
    pub body: String,
}

impl Message {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }
}

impl From<Message> for fcm_sdk::Message {
    fn from(Message { title, body }: Message) -> Self {
        Self { title, body }
    }
}

#[derive(Component)]
#[lifecycle]
pub struct SendNotification {
    fcm_client: Option<FirebaseCloudMessaging>,
}

impl SendNotification {
    pub fn fcm_client(&self) -> &FirebaseCloudMessaging {
        self.fcm_client.as_ref().unwrap()
    }
}

#[async_trait::async_trait]
impl ComponentLifecycle for SendNotification {
    async fn start(&mut self) {
        let fcm_client = FirebaseCloudMessaging::from_env();

        self.fcm_client.replace(fcm_client);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[async_trait::async_trait]
impl<T> Command<(Vec<String>, T), ()> for SendNotification
where
    T: Into<fcm_sdk::Message> + Send + Sync + 'static,
{
    type Error = crate::Error;

    async fn execute(&self, (tokens, message): (Vec<String>, T)) -> Result<(), Self::Error> {
        let message = message.into();

        // TODO: error handle
        self.fcm_client()
            .send_to_devices(tokens, message)
            .await
            .ok();

        Ok(())
    }
}

pub mod r#trait {
    use crate::command::r#trait::Command;

    pub trait SendMessage: Command<(), ()> {}
}
