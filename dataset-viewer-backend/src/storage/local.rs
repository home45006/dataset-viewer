use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

use crate::storage::{
    ConnectionConfig, DirectoryResult, FileInfo, ListOptions, StorageClient, StorageError,
};

/// 本地文件系统客户端
pub struct LocalClient {
    config: ConnectionConfig,
    connected: bool,
    root_path: Option<String>,
}

impl LocalClient {
    pub fn new(config: ConnectionConfig) -> Result<Self, StorageError> {
        Ok(Self {
            config,
            connected: false,
            root_path: None,
        })
    }

    /// 解析完整路径
    fn resolve_path(&self, path: &str) -> Result<std::path::PathBuf, StorageError> {
        let path = path.trim_start_matches('/');
        
        if let Some(root) = &self.root_path {
            Ok(Path::new(root).join(path))
        } else {
            Ok(Path::new(path).to_path_buf())
        }
    }

    /// 格式化文件信息
    async fn format_file_info(&self, path: &std::path::Path, base_path: &std::path::Path) -> Result<FileInfo, StorageError> {
        let metadata = fs::metadata(path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        let relative_path = path.strip_prefix(base_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        let filename = relative_path.to_string_lossy().to_string();
        let basename = path.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_string_lossy()
            .to_string();

        let file_type = if metadata.is_dir() { "directory" } else { "file" };
        let size = metadata.len().to_string();
        
        let lastmod = metadata
            .modified()
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

        // 尝试推断 MIME 类型
        let mime = if metadata.is_file() {
            path.extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| match ext.to_lowercase().as_str() {
                    "txt" => Some("text/plain".to_string()),
                    "json" => Some("application/json".to_string()),
                    "csv" => Some("text/csv".to_string()),
                    "parquet" => Some("application/octet-stream".to_string()),
                    "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
                    "zip" => Some("application/zip".to_string()),
                    "tar" => Some("application/x-tar".to_string()),
                    "gz" => Some("application/gzip".to_string()),
                    "jpg" | "jpeg" => Some("image/jpeg".to_string()),
                    "png" => Some("image/png".to_string()),
                    "pdf" => Some("application/pdf".to_string()),
                    _ => Some("application/octet-stream".to_string()),
                })
        } else {
            None
        };

        Ok(FileInfo {
            filename,
            basename,
            lastmod,
            size,
            file_type: file_type.to_string(),
            mime,
            etag: None,
        })
    }
}

#[async_trait]
impl StorageClient for LocalClient {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), StorageError> {
        // 验证配置
        self.validate_config(config)?;
        
        // 设置根路径
        self.root_path = config.root_path.clone().or_else(|| config.url.clone());
        
        // 检查根路径是否存在
        if let Some(root) = &self.root_path {
            let path = Path::new(root);
            if !path.exists() {
                return Err(StorageError::NotFound(format!("根路径不存在: {}", root)));
            }
            if !path.is_dir() {
                return Err(StorageError::InvalidConfig(format!("根路径不是目录: {}", root)));
            }
        }

        self.connected = true;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<(), StorageError> {
        self.connected = false;
        self.root_path = None;
        Ok(())
    }

    async fn list_directory(
        &self,
        path: &str,
        options: Option<&ListOptions>,
    ) -> Result<DirectoryResult, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        let dir_path = self.resolve_path(path)?;
        
        if !dir_path.exists() {
            return Err(StorageError::NotFound(format!("目录不存在: {}", path)));
        }

        if !dir_path.is_dir() {
            return Err(StorageError::InvalidConfig(format!("路径不是目录: {}", path)));
        }

        let mut files = Vec::new();
        let mut entries = fs::read_dir(&dir_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| StorageError::IoError(e.to_string()))? {
            
            match self.format_file_info(&entry.path(), &dir_path).await {
                Ok(file_info) => files.push(file_info),
                Err(e) => {
                    tracing::warn!("格式化文件信息失败: {}", e);
                    continue;
                }
            }
        }

        // 应用排序
        if let Some(opts) = options {
            if let Some(sort_by) = &opts.sort_by {
                match sort_by.as_str() {
                    "name" => {
                        files.sort_by(|a, b| {
                            let cmp = a.filename.cmp(&b.filename);
                            if opts.sort_order.as_deref() == Some("desc") {
                                cmp.reverse()
                            } else {
                                cmp
                            }
                        });
                    }
                    "size" => {
                        files.sort_by(|a, b| {
                            let a_size: u64 = a.size.parse().unwrap_or(0);
                            let b_size: u64 = b.size.parse().unwrap_or(0);
                            let cmp = a_size.cmp(&b_size);
                            if opts.sort_order.as_deref() == Some("desc") {
                                cmp.reverse()
                            } else {
                                cmp
                            }
                        });
                    }
                    "modified" => {
                        files.sort_by(|a, b| {
                            let cmp = a.lastmod.cmp(&b.lastmod);
                            if opts.sort_order.as_deref() == Some("desc") {
                                cmp.reverse()
                            } else {
                                cmp
                            }
                        });
                    }
                    _ => {}
                }
            }
        }

        // 应用分页
        let (files, has_more) = if let Some(opts) = options {
            if let Some(page_size) = opts.page_size {
                let start = opts.marker.as_deref()
                    .and_then(|m| m.parse::<usize>().ok())
                    .unwrap_or(0);
                let end = start + page_size as usize;
                
                let paginated = files.get(start..end.min(files.len())).unwrap_or(&[]).to_vec();
                let has_more = end < files.len();
                
                (paginated, has_more)
            } else {
                (files, false)
            }
        } else {
            (files, false)
        };

        Ok(DirectoryResult {
            files,
            has_more,
            next_marker: if has_more {
                Some((files.len()).to_string())
            } else {
                None
            },
            total_count: None,
            path: path.to_string(),
        })
    }

    async fn read_file_range(
        &self,
        path: &str,
        start: u64,
        length: u64,
    ) -> Result<Vec<u8>, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        let file_path = self.resolve_path(path)?;
        
        if !file_path.exists() {
            return Err(StorageError::NotFound(format!("文件不存在: {}", path)));
        }

        if !file_path.is_file() {
            return Err(StorageError::InvalidConfig(format!("路径不是文件: {}", path)));
        }

        use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
        
        let mut file = fs::File::open(&file_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        file.seek(SeekFrom::Start(start)).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut buffer = vec![0u8; length as usize];
        let bytes_read = file.read(&mut buffer).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    async fn read_full_file(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        let file_path = self.resolve_path(path)?;
        
        if !file_path.exists() {
            return Err(StorageError::NotFound(format!("文件不存在: {}", path)));
        }

        if !file_path.is_file() {
            return Err(StorageError::InvalidConfig(format!("路径不是文件: {}", path)));
        }

        fs::read(&file_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))
    }

    async fn get_file_size(&self, path: &str) -> Result<u64, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        let file_path = self.resolve_path(path)?;
        
        if !file_path.exists() {
            return Err(StorageError::NotFound(format!("文件不存在: {}", path)));
        }

        let metadata = fs::metadata(&file_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(metadata.len())
    }

    fn protocol(&self) -> &str {
        "local"
    }

    fn validate_config(&self, config: &ConnectionConfig) -> Result<(), StorageError> {
        if config.protocol != "local" && config.protocol != "file" {
            return Err(StorageError::ProtocolNotSupported(config.protocol.clone()));
        }
        Ok(())
    }

    fn build_protocol_url(&self, path: &str) -> String {
        let resolved_path = self.resolve_path(path).unwrap_or_else(|_| path.into());
        format!("file://{}", resolved_path.display())
    }
}