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

    /// 获取客户端
    pub async fn get_client(&self, session_id: &str) -> Result<Arc<RwLock<Box<dyn StorageClient>>>, StorageError> {
        let connections = self.connections.read().await;
        if connections.contains_key(session_id) {
            // 这里需要返回一个可以安全共享的客户端引用
            // 由于 trait object 的限制，我们需要重新设计这部分
            Err(StorageError::NotConnected)
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
}