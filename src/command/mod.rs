pub mod send_notification;

use fcm_sdk::Message;
use sai::{Component, Injected};

use self::{r#trait::Command, send_notification::SendNotification};

pub mod r#trait {

    /// 인자가 여러개라면 Command<(String, u8, i8, u32), String> 이런식으로
    #[async_trait::async_trait]
    pub trait Command<T, R> {
        type Error;

        async fn execute(&self, _: T) -> Result<R, Self::Error>;
    }
}

#[derive(Component)]
pub struct CommandSet {
    #[injected]
    send_notification: Injected<SendNotification>,
}

impl CommandSet {
    pub async fn send_notification(
        &self,
        tokens: Vec<String>,
        message: impl Into<Message> + Send + Sync + 'static,
    ) -> crate::Result<()> {
        self.send_notification.execute((tokens, message)).await
    }
}

#[cfg(test)]
pub mod tests {
    /* pub use super::get_user_info::tests::*; */
}
