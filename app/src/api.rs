use serde::Serialize;

#[derive(Serialize)]
pub struct Status {
    pub ip: String,
    pub services: Vec<Service>,
}

#[derive(Serialize)]
pub struct Service {
    pub ip: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct Error {
    pub error: String,
}
