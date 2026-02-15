#[derive(Clone)]
pub struct CoordinatesConfig {
    origin_space : String,        // Space of origin for projection
    destination_space : String,   // Destination space of projection
}


impl CoordinatesConfig {
    pub fn new(origin_space : String, destination_space : String) -> Self {
        CoordinatesConfig {
            origin_space : origin_space,
            destination_space : destination_space,
        }
    }

    pub fn get_origin_space(&self) -> &String {
        &self.origin_space
    }

    pub fn get_destination_space(&self) -> &String {
        &self.destination_space
    }
}