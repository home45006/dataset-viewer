use crate::{
    config::Config,
    storage::{
        huggingface::HuggingFaceClient, local::LocalClient, oss::OSSClient, smb::SMBClient,
        ssh::SSHClient, webdav::WebDAVClient, ConnectionConfig, StorageClient, StorageError,
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

/// 存储管理器，负责管理所有存储客户端的连接和生命周期
pub struct StorageManager {
    config: Config,
    connections: Arc<RwLock<HashMap<String, Box<dyn StorageClient>>>>,
    active_session: Arc<RwLock<Option<String>>>,
}

impl StorageManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    /// 创建存储客户端
    pub async fn create_client(&self, config: &ConnectionConfig) -> Result<String, StorageError> {
        let session_id = Uuid::new_v4().to_string();
        
        let mut client: Box<dyn StorageClient> = match config.protocol.as_str() {
            "local" | "file" => Box::new(LocalClient::new(config.clone())?),
            "oss" | "s3" => Box::new(OSSClient::new(config.clone())?),
            "webdav" | "webdavs" => Box::new(WebDAVClient::new(config.clone())?),
            "ssh" | "sftp" => Box::new(SSHClient::new(config.clone())?),
            "smb" | "cifs" => Box::new(SMBClient::new(config.clone())?),
            "huggingface" => Box::new(HuggingFaceClient::new(config.clone())?),
            protocol => {
                return Err(StorageError::ProtocolNotSupported(format!(
                    "不支持的协议: {}",
                    protocol
                )))
            }
        };

        // 尝试连接
        client.connect(config).await?;

        // 存储连接
        {
            let mut connections = self.connections.write().await;
            connections.insert(session_id.clone(), client);
        }

        // 设置为活动会话
        {
            let mut active_session = self.active_session.write().await;
            *active_session = Some(session_id.clone());
        }

        Ok(session_id)
    }

    /// 获取客户端引用（临时实现，用于目录列表功能）
    pub async fn with_client<T, F>(&self, session_id: &str, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&dyn StorageClient) -> T,
    {
        let connections = self.connections.read().await;
        if let Some(client) = connections.get(session_id) {
            Ok(f(client.as_ref()))
        } else {
            Err(StorageError::NotConnected)
        }
    }

    /// 列出目录内容的便利方法
    pub async fn list_directory(
        &self,
        session_id: &str,
        path: &str,
        options: Option<&crate::storage::ListOptions>,
    ) -> Result<crate::storage::DirectoryResult, StorageError> {
        let connections = self.connections.read().await;
        if let Some(client) = connections.get(session_id) {
            client.list_directory(path, options).await
        } else {
            Err(StorageError::NotConnected)
        }
    }

    /// 获取活动客户端
    pub async fn get_active_client(&self) -> Result<String, StorageError> {
        let active_session = self.active_session.read().await;
        active_session
            .clone()
            .ok_or(StorageError::NotConnected)
    }

    /// 断开连接
    pub async fn disconnect(&self, session_id: &str) -> Result<(), StorageError> {
        let mut connections = self.connections.write().await;
        if let Some(mut client) = connections.remove(session_id) {
            client.disconnect().await?;
        }

        // 如果是活动会话，清除活动会话
        {
            let mut active_session = self.active_session.write().await;
            if let Some(active) = &*active_session {
                if active == session_id {
                    *active_session = None;
                }
            }
        }

        Ok(())
    }

    /// 断开所有连接
    pub async fn disconnect_all(&self) -> Result<(), StorageError> {
        let mut connections = self.connections.write().await;
        
        for (_, mut client) in connections.drain() {
            let _ = client.disconnect().await; // 忽略断开连接时的错误
        }

        {
            let mut active_session = self.active_session.write().await;
            *active_session = None;
        }

        Ok(())
    }

    /// 获取所有连接的会话ID
    pub async fn list_sessions(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }

    /// 检查会话是否存在
    pub async fn session_exists(&self, session_id: &str) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(session_id)
    }

    /// 获取文件内容的便利方法
    pub async fn get_file_content(
        &self,
        session_id: &str,
        path: &str,
        start: Option<u64>,
        length: Option<u64>,
    ) -> Result<crate::storage::FileContent, StorageError> {
        let connections = self.connections.read().await;
        if let Some(client) = connections.get(session_id) {
            client.get_file_content(path, start, length).await
        } else {
            Err(StorageError::NotConnected)
        }
    }

    /// 获取会话客户端
    pub async fn get_session_client(&self, session_id: &str) -> Option<Arc<dyn StorageClient>> {
        let connections = self.connections.read().await;
        if let Some(client) = connections.get(session_id) {
            // 注意：这里需要克隆 Arc 引用，但是 Box<dyn StorageClient> 不能直接转换为 Arc<dyn StorageClient>
            // 我们需要一个不同的方法
            None
        } else {
            None
        }
    }

    /// 获取文件大小的便利方法
    pub async fn get_file_size(
        &self,
        session_id: &str,
        path: &str,
    ) -> Result<u64, StorageError> {
        let connections = self.connections.read().await;
        if let Some(client) = connections.get(session_id) {
            client.get_file_size(path).await
        } else {
            Err(StorageError::NotConnected)
        }
    }

    /// 读取文件范围的便利方法
    pub async fn read_file_range(
        &self,
        session_id: &str,
        path: &str,
        start: u64,
        length: u64,
    ) -> Result<Vec<u8>, StorageError> {
        let connections = self.connections.read().await;
        if let Some(client) = connections.get(session_id) {
            client.read_file_range(path, start, length).await
        } else {
            Err(StorageError::NotConnected)
        }
    }
}