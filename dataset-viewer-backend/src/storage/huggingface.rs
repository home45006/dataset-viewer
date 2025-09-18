use async_trait::async_trait;
use reqwest::Client;

use crate::storage::{
    ConnectionConfig, DirectoryResult, ListOptions, StorageClient, StorageError,
};

/// HuggingFace Hub 客户端
pub struct HuggingFaceClient {
    client: Client,
    config: ConnectionConfig,
    connected: bool,
    api_token: Option<String>,
}

impl HuggingFaceClient {
    pub fn new(config: ConnectionConfig) -> Result<Self, StorageError> {
        let api_token = config.password.clone(); // 使用 password 字段存储 API token

        Ok(Self {
            client: Client::new(),
            config,
            connected: false,
            api_token,
        })
    }
}

#[async_trait]
impl StorageClient for HuggingFaceClient {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), StorageError> {
        self.validate_config(config)?;
        // TODO: 实现 HuggingFace API 连接测试
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

    async fn list_directory(&self, path: &str, _options: Option<&ListOptions>) -> Result<DirectoryResult, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }
        // TODO: 实现 HuggingFace Hub API 调用
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
        "huggingface"
    }

    fn validate_config(&self, config: &ConnectionConfig) -> Result<(), StorageError> {
        if config.protocol != "huggingface" {
            return Err(StorageError::ProtocolNotSupported(config.protocol.clone()));
        }
        Ok(())
    }

    fn build_protocol_url(&self, path: &str) -> String {
        format!("huggingface://{}", path)
    }
}