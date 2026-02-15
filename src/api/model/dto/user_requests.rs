use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GetUserRequest {
    pub uuid: Option<Uuid>,
    pub email: Option<String>
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


