use uuid::Uuid;

#[derive(Debug)]
pub struct FcmToken {
    pub udid: Uuid,
    pub user_id: Uuid,
    pub fcm_token: String,
}

impl FcmToken {
    pub fn new(udid: Uuid, user_id: Uuid, fcm_token: String) -> Self {
        Self {
            udid,
            user_id,
            fcm_token,
        }
    }
}
