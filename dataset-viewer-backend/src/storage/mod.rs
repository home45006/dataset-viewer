pub mod manager;
pub mod traits;
pub mod types;

// 各种存储客户端
pub mod local;
pub mod oss;
pub mod webdav;
pub mod ssh;
pub mod smb;
pub mod huggingface;

// 重新导出主要类型
pub use manager::StorageManager;
pub use traits::StorageClient;
pub use types::*;