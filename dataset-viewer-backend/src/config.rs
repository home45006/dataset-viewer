use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub session_timeout_minutes: u64,
    pub max_file_size_mb: u64,
    pub chunk_size_kb: u64,
    pub max_concurrent_downloads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allow_local_files: bool,
    pub max_archive_entries: usize,
    pub max_preview_size_mb: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_connections: 1000,
                request_timeout_seconds: 30,
            },
            storage: StorageConfig {
                session_timeout_minutes: 60,
                max_file_size_mb: 1024, // 1GB
                chunk_size_kb: 1024,    // 1MB
                max_concurrent_downloads: 10,
            },
            security: SecurityConfig {
                allow_local_files: true,
                max_archive_entries: 10000,
                max_preview_size_mb: 10,
            },
        }
    }
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::default();

        // 从环境变量覆盖配置
        if let Ok(host) = env::var("SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port_str) = env::var("SERVER_PORT") {
            config.server.port = port_str.parse()?;
        }

        if let Ok(allow_local) = env::var("ALLOW_LOCAL_FILES") {
            config.security.allow_local_files = allow_local.to_lowercase() == "true";
        }

        Ok(config)
    }
}