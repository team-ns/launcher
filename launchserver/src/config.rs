use crate::config::auth::{JsonAuthConfig, SqlAuthConfig};
use anyhow::Result;
use launcher_api::config::Configurable;
use serde::{Deserialize, Serialize};

use crate::auth::accept::AcceptAuthProvider;
use crate::auth::json::JsonAuthProvider;
use crate::auth::sql::SqlAuthProvider;
use crate::auth::AuthProvider;
use std::clone::Clone;
use std::path::Path;

pub(crate) mod auth;
mod texture;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub bind_address: String,
    pub auth: AuthConfig,
    pub texture: TextureProvider,
    pub file_server: String,
    pub websocket_url: String,
    pub project_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextureProvider {
    skin_url: String,
    cape_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AuthConfig {
    JSON(JsonAuthConfig),
    SQL(SqlAuthConfig),
    Accept,
}

impl AuthConfig {
    pub fn get_provider(&self) -> Result<AuthProvider> {
        let provider = match self {
            AuthConfig::JSON(config) => AuthProvider::Json(JsonAuthProvider::new(config.clone())?),
            AuthConfig::SQL(config) => AuthProvider::Sql(SqlAuthProvider::new(config.clone())?),
            AuthConfig::Accept => AuthProvider::Accept(AcceptAuthProvider::default()),
        };
        Ok(provider)
    }
}

#[teloc::inject]
impl Config {
    pub fn new() -> Self {
        log::info!("Read config file");
        Self::get_config(Path::new("config.json")).expect("Can't read config file!")
    }
}

impl Configurable for Config {}

impl Default for Config {
    fn default() -> Self {
        Config {
            file_server: "http://127.0.0.1:8080/files".to_string(),
            bind_address: "127.0.0.1:8080".to_string(),
            auth: AuthConfig::Accept,
            texture: TextureProvider {
                skin_url: "http://example.com/skin/{username}.png".to_string(),
                cape_url: "http://example.com/cape/{username}.png".to_string(),
            },
            websocket_url: "ws://127.0.0.1:8080".to_string(),
            project_name: "NSL".to_string(),
        }
    }
}
