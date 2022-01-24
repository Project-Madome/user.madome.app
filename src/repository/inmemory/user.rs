use std::sync::Mutex;

use sai::Component;

use crate::{entity::User, repository::r#trait::UserRepository};

#[derive(Component)]
pub struct InMemoryUserRepository {
    inner: Mutex<Vec<User>>,
}

#[async_trait::async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn add(&self, user: User) -> crate::Result<Option<User>> {
        let mut inner = self.inner.lock().unwrap();

        let already_user = inner.iter().any(|exist| {
            exist.id == user.id || exist.email == user.email || exist.name == user.name
        });

        if already_user {
            return Ok(None);
        }

        inner.push(user.clone());

        Ok(Some(user))
    }

    async fn get(&self, id_or_email: String) -> crate::Result<Option<User>> {
        let inner = self.inner.lock().unwrap();

        let user = inner
            .iter()
            .find(|user| user.id.to_string() == id_or_email || user.email == id_or_email)
            .cloned();

        Ok(user)
    }
}
