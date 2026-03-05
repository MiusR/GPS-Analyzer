use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetEventsRequest {
    pub name: Option<String>
}

#[derive(Deserialize)]
pub struct DeleteEventRequest {
    pub name : String
}

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub name: String
}



