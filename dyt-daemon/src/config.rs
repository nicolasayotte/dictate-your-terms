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
    std::thread::available_parallelism()
        .map(|n| n.get() as u16)
        .unwrap_or(4)
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

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn defaults_applied() {
        let toml_str = "[server]\n[engine]\nmodel_path = \"x\"\n";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3030);
        assert_eq!(config.engine.provider, "whisper_cpp");
        assert!(config.engine.threads >= 1, "threads must be at least 1");
        assert_eq!(config.engine.model_path, "x");
    }

    #[test]
    fn full_config_parsed() {
        let toml_str = r#"
[server]
host = "0.0.0.0"
port = 9090

[engine]
provider = "onnx"
model_path = "/models/large.bin"
threads = 8
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.engine.provider, "onnx");
        assert_eq!(config.engine.model_path, "/models/large.bin");
        assert_eq!(config.engine.threads, 8);
    }

    #[test]
    fn invalid_toml_errors() {
        let bad = "this is not valid toml ][[[";
        let result = toml::from_str::<Config>(bad);
        assert!(result.is_err(), "expected Err for malformed TOML, got Ok");
    }
}
