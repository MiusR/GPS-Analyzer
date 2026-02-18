use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::model::{auth::oauth::OAuthProvider, user::User};

#[derive(Deserialize)]
pub struct GetUserRequest {
    pub uuid: Option<Uuid>,
    pub email: Option<String>,
    pub provider : Option<OAuthProvider>,
    pub provider_user_id: Option<String>
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email : String,
    pub tier : String
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub uuid : Uuid,
    pub name: Option<String>,
    pub email : Option<String>,
    pub tier : Option<String>
}

#[derive(Deserialize)]
pub struct DeleteUserRequest {
    pub uuid : Uuid
}


#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub provider: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        UserResponse {
            id: user.get_uuid().to_string(),
            provider: user.get_provider().to_string(),
            email: Some(user.get_email().to_string()),
            name: Some(user.get_name().to_string()), // FIXME : this might not be provided by authentication services so add some to user
            avatar_url: user.get_avatar_url().clone(),
        }
    }
}

