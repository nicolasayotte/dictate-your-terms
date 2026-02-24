use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub engine: EngineConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    pub model_path: String,
    #[serde(default = "default_threads")]
    pub threads: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3030
}

fn default_provider() -> String {
    "whisper_cpp".to_string()
}

fn default_threads() -> u16 {
    4
}

/// Load config from the standard path: ~/.config/dyt/config.toml
pub fn load() -> Result<Config> {
    let path = config_path();
    let contents =
        std::fs::read_to_string(&path).with_context(|| format!("reading config at {path:?}"))?;
    toml::from_str(&contents).with_context(|| "parsing config.toml")
}

fn config_path() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        dir.join("dyt").join("config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}
