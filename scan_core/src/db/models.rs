use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Device {
    pub address: String,
    pub username: String,
    pub password: String,
}
