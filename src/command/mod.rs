use sai::Component;

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
    /* #[cfg(not(test))]
#[injected]
get_user_info: Injected<GetUserInfo>,

#[cfg(test)]
#[injected]
get_user_info: Injected<tests::GetUserInfo>, */}

impl CommandSet {
    /* pub async fn get_user_info(&self, user_id_or_email: String) -> crate::Result<UserInfo> {
        self.get_user_info.execute(user_id_or_email).await
    } */
}

#[cfg(test)]
pub mod tests {
    /* pub use super::get_user_info::tests::*; */
}
