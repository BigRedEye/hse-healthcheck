use crate::prelude::*;
use config;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub bind_address: std::net::SocketAddr,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut s = config::Config::new();
        s.merge(config::Environment::with_prefix("node"))?;
        s.try_into().map_err(|e| anyhow::Error::new(e))
    }
}
