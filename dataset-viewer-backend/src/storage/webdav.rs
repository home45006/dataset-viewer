use async_trait::async_trait;
use reqwest::Client;

use crate::storage::{
    ConnectionConfig, DirectoryResult, ListOptions, StorageClient, StorageError,
};

/// WebDAV 客户端
pub struct WebDAVClient {
    client: Client,
    config: ConnectionConfig,
    connected: bool,
    base_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl WebDAVClient {
    pub fn new(config: ConnectionConfig) -> Result<Self, StorageError> {
        let base_url = config
            .url
            .clone()
            .ok_or_else(|| StorageError::InvalidConfig("WebDAV URL is required".to_string()))?;

        Ok(Self {
            client: Client::new(),
            config,
            connected: false,
            base_url,
            username: config.username.clone(),
            password: config.password.clone(),
        })
    }
}

#[async_trait]
impl StorageClient for WebDAVClient {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), StorageError> {
        self.validate_config(config)?;
        // TODO: 实现 WebDAV 连接测试
        self.connected = true;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<(), StorageError> {
        self.connected = false;
        Ok(())
    }

    async fn list_directory(
        &self,
        path: &str,
        _options: Option<&ListOptions>,
    ) -> Result<DirectoryResult, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        // TODO: 实现 WebDAV PROPFIND 请求
        Ok(DirectoryResult {
            files: vec![],
            has_more: false,
            next_marker: None,
            total_count: None,
            path: path.to_string(),
        })
    }

    async fn read_file_range(&self, _path: &str, _start: u64, _length: u64) -> Result<Vec<u8>, StorageError> {
        Err(StorageError::RequestFailed("Not implemented".to_string()))
    }

    async fn read_full_file(&self, _path: &str) -> Result<Vec<u8>, StorageError> {
        Err(StorageError::RequestFailed("Not implemented".to_string()))
    }

    async fn get_file_size(&self, _path: &str) -> Result<u64, StorageError> {
        Err(StorageError::RequestFailed("Not implemented".to_string()))
    }

    fn protocol(&self) -> &str {
        "webdav"
    }

    fn validate_config(&self, config: &ConnectionConfig) -> Result<(), StorageError> {
        if config.protocol != "webdav" && config.protocol != "webdavs" {
            return Err(StorageError::ProtocolNotSupported(config.protocol.clone()));
        }
        if config.url.is_none() {
            return Err(StorageError::InvalidConfig("WebDAV URL is required".to_string()));
        }
        Ok(())
    }

    fn build_protocol_url(&self, path: &str) -> String {
        let clean_path = path.trim_start_matches('/');
        if clean_path.is_empty() {
            self.base_url.clone()
        } else {
            format!("{}/{}", self.base_url.trim_end_matches('/'), clean_path)
        }
    }
}