use std::env;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;

pub const DEFAULT_OPENROUTER_MODEL: &str = "nvidia/nemotron-3-super-120b-a12b:free";

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: String,
    pub openrouter_http_referer: Option<String>,
    pub openrouter_title: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bind_addr: env::var("BACKEND_BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_owned()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:gpt-copy-v6.db".to_owned()),
            openrouter_api_key: non_empty_env("OPENROUTER_API_KEY"),
            openrouter_model: non_empty_env("OPENROUTER_MODEL")
                .unwrap_or_else(|| DEFAULT_OPENROUTER_MODEL.to_owned()),
            openrouter_http_referer: non_empty_env("OPENROUTER_HTTP_REFERER"),
            openrouter_title: non_empty_env("OPENROUTER_TITLE")
                .or_else(|| non_empty_env("X_OPENROUTER_TITLE"))
                .unwrap_or_else(|| "gpt-copy-v6".to_owned()),
        }
    }

    pub fn for_tests() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".to_owned(),
            database_url: "sqlite::memory:".to_owned(),
            openrouter_api_key: None,
            openrouter_model: DEFAULT_OPENROUTER_MODEL.to_owned(),
            openrouter_http_referer: None,
            openrouter_title: "gpt-copy-v6-tests".to_owned(),
        }
    }

    pub fn bind_socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        let bind_addr = self.bind_addr.parse::<SocketAddr>().map_err(|source| {
            ConfigError(format!(
                "BACKEND_BIND_ADDR must be a socket address: {source}"
            ))
        })?;

        if !bind_addr.ip().is_loopback() {
            return Err(ConfigError(format!(
                "BACKEND_BIND_ADDR must bind to a loopback address unless authentication is configured: {bind_addr}"
            )));
        }

        Ok(bind_addr)
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ConfigError {}
