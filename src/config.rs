use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub admin_host: String,
    pub default_redirect_host: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let admin_host = env::var("ADMIN_HOST").unwrap_or_else(|_| "lynx".to_string());
        let default_redirect_host = env::var("DEFAULT_REDIRECT_HOST").unwrap_or_else(|_| "go".to_string());
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable is required"))?;

        Ok(Self {
            admin_host,
            default_redirect_host,
            database_url,
        })
    }
}
