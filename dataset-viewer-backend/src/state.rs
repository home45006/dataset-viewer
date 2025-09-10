use crate::{
    config::Config,
    storage::StorageManager,
    websocket::WebSocketManager,
    Result,
};
use std::sync::Arc;

/// 应用状态，包含所有共享的资源和管理器
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub storage_manager: Arc<StorageManager>,
    pub websocket_manager: Arc<WebSocketManager>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let storage_manager = Arc::new(StorageManager::new(config.clone()));
        let websocket_manager = Arc::new(WebSocketManager::new());

        Ok(Self {
            config,
            storage_manager,
            websocket_manager,
        })
    }
}