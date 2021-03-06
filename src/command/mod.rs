pub mod has_book;
pub mod has_book_tag;
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
    #[allow(dead_code)]
    #[injected]
    send_notification: Injected<SendNotification>,

    #[injected]
    has_book: Injected<has_book::HasBook>,

    #[injected]
    has_book_tag: Injected<has_book_tag::HasBookTag>,
}

impl CommandSet {
    #[allow(dead_code)]
    pub async fn send_notification(
        &self,
        tokens: Vec<String>,
        message: impl Into<Message> + Send + Sync + 'static,
    ) -> crate::Result<()> {
        self.send_notification.execute((tokens, message)).await
    }

    pub async fn has_book(&self, book_id: u32) -> crate::Result<bool> {
        self.has_book.execute(book_id).await
    }

    pub async fn has_book_tag(
        &self,
        tag_kind: impl Into<String>,
        tag_name: impl Into<String>,
    ) -> crate::Result<bool> {
        self.has_book_tag
            .execute((tag_kind.into(), tag_name.into()))
            .await
    }
}

#[cfg(test)]
pub mod tests {
    /* pub use super::get_user_info::tests::*; */
}
