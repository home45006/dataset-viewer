use serde::{Deserialize, Serialize};

/// 压缩包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    pub entries: Vec<ArchiveEntry>,
    pub total_entries: u64,
    pub total_uncompressed_size: u64,
    pub total_compressed_size: u64,
    pub format: ArchiveFormat,
    pub has_more: bool,
}

/// 压缩包条目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub modified: Option<String>,
    pub is_directory: bool,
    pub is_encrypted: bool,
    pub crc32: Option<u32>,
}

/// 压缩包格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    SevenZ,
    Rar,
    Gzip,
    Unknown,
}

impl ArchiveFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "zip" => Self::Zip,
            "tar" => Self::Tar,
            "tgz" | "tar.gz" => Self::TarGz,
            "tbz" | "tbz2" | "tar.bz2" => Self::TarBz2,
            "txz" | "tar.xz" => Self::TarXz,
            "7z" => Self::SevenZ,
            "rar" => Self::Rar,
            "gz" => Self::Gzip,
            _ => Self::Unknown,
        }
    }
}

/// 文件预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub content: Vec<u8>,
    pub is_truncated: bool,
    pub total_size: u64,
    pub preview_size: u64,
}

/// 压缩包处理错误
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("不支持的压缩格式: {0}")]
    UnsupportedFormat(String),

    #[error("压缩包损坏或无效: {0}")]
    InvalidArchive(String),

    #[error("文件在压缩包中未找到: {0}")]
    FileNotFound(String),

    #[error("解压缩失败: {0}")]
    ExtractionFailed(String),

    #[error("IO 错误: {0}")]
    IoError(String),

    #[error("权限错误: {0}")]
    PermissionDenied(String),

    #[error("压缩包过大或条目过多")]
    TooLarge,
}