use crate::archive::{ArchiveEntry, ArchiveError, ArchiveFormat, ArchiveInfo, FilePreview};
use std::io::{Read, Seek};

/// 压缩包处理器
pub struct ArchiveHandler;

impl ArchiveHandler {
    /// 分析压缩包信息
    pub async fn analyze<R>(
        reader: R,
        format: ArchiveFormat,
        max_entries: Option<usize>,
    ) -> Result<ArchiveInfo, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        match format {
            ArchiveFormat::Zip => Self::analyze_zip(reader, max_entries).await,
            ArchiveFormat::Tar => Self::analyze_tar(reader, max_entries).await,
            ArchiveFormat::TarGz => Self::analyze_tar_gz(reader, max_entries).await,
            _ => Err(ArchiveError::UnsupportedFormat(format!("{:?}", format))),
        }
    }

    /// 从压缩包中提取文件内容
    pub async fn extract_file<R>(
        reader: R,
        format: ArchiveFormat,
        file_path: &str,
        max_size: Option<u64>,
    ) -> Result<FilePreview, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        match format {
            ArchiveFormat::Zip => Self::extract_file_from_zip(reader, file_path, max_size).await,
            ArchiveFormat::Tar => Self::extract_file_from_tar(reader, file_path, max_size).await,
            ArchiveFormat::TarGz => Self::extract_file_from_tar_gz(reader, file_path, max_size).await,
            _ => Err(ArchiveError::UnsupportedFormat(format!("{:?}", format))),
        }
    }

    /// 分析 ZIP 压缩包
    async fn analyze_zip<R>(
        mut reader: R,
        max_entries: Option<usize>,
    ) -> Result<ArchiveInfo, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        // TODO: 使用 tokio::task::spawn_blocking 在后台线程中处理
        tokio::task::spawn_blocking(move || {
            use zip::ZipArchive;

            let mut archive = ZipArchive::new(reader)
                .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;

            let total_entries = archive.len() as u64;
            let limit = max_entries.unwrap_or(total_entries as usize);
            let mut entries = Vec::new();
            let mut total_uncompressed_size = 0u64;
            let mut total_compressed_size = 0u64;

            for i in 0..limit.min(archive.len()) {
                let file = archive.by_index(i)
                    .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;

                let entry = ArchiveEntry {
                    path: file.name().to_string(),
                    name: std::path::Path::new(file.name())
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| file.name().to_string()),
                    size: file.size(),
                    compressed_size: Some(file.compressed_size()),
                    modified: file.last_modified()
                        .and_then(|dt| dt.to_time().ok())
                        .and_then(|t| chrono::DateTime::<chrono::Utc>::from_timestamp(t.unix_timestamp(), 0))
                        .map(|dt| dt.to_rfc3339()),
                    is_directory: file.is_dir(),
                    is_encrypted: file.encrypted(),
                    crc32: Some(file.crc32()),
                };

                total_uncompressed_size += file.size();
                total_compressed_size += file.compressed_size();
                entries.push(entry);
            }

            Ok(ArchiveInfo {
                entries,
                total_entries,
                total_uncompressed_size,
                total_compressed_size,
                format: ArchiveFormat::Zip,
                has_more: limit < archive.len(),
            })
        })
        .await
        .map_err(|e| ArchiveError::IoError(e.to_string()))?
    }

    /// 从 ZIP 中提取文件
    async fn extract_file_from_zip<R>(
        mut reader: R,
        file_path: &str,
        max_size: Option<u64>,
    ) -> Result<FilePreview, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        let file_path = file_path.to_string();
        
        tokio::task::spawn_blocking(move || {
            use zip::ZipArchive;

            let mut archive = ZipArchive::new(reader)
                .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;

            let mut file = archive.by_name(&file_path)
                .map_err(|_| ArchiveError::FileNotFound(file_path))?;

            let total_size = file.size();
            let read_size = max_size.unwrap_or(total_size).min(total_size);
            
            let mut buffer = vec![0u8; read_size as usize];
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| ArchiveError::ExtractionFailed(e.to_string()))?;

            buffer.truncate(bytes_read);

            Ok(FilePreview {
                content: buffer,
                is_truncated: bytes_read < total_size as usize,
                total_size,
                preview_size: bytes_read as u64,
            })
        })
        .await
        .map_err(|e| ArchiveError::IoError(e.to_string()))?
    }

    /// 分析 TAR 压缩包
    async fn analyze_tar<R>(
        reader: R,
        max_entries: Option<usize>,
    ) -> Result<ArchiveInfo, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            use tar::Archive;

            let mut archive = Archive::new(reader);
            let mut entries = Vec::new();
            let mut total_entries = 0u64;
            let mut total_uncompressed_size = 0u64;
            let limit = max_entries.unwrap_or(usize::MAX);

            for (index, entry) in archive.entries()
                .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
                .enumerate()
            {
                if index >= limit {
                    break;
                }

                let entry = entry.map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;
                let header = entry.header();

                let path = entry.path()
                    .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
                    .to_string_lossy()
                    .to_string();

                let name = std::path::Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());

                let size = header.size().unwrap_or(0);
                let modified = header.mtime()
                    .ok()
                    .and_then(|timestamp| chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0))
                    .map(|dt| dt.to_rfc3339());

                let entry_info = ArchiveEntry {
                    path,
                    name,
                    size,
                    compressed_size: None, // TAR不压缩，所以没有压缩大小
                    modified,
                    is_directory: header.entry_type().is_dir(),
                    is_encrypted: false, // TAR不支持加密
                    crc32: None, // TAR通常不存储CRC32
                };

                total_uncompressed_size += size;
                entries.push(entry_info);
                total_entries += 1;
            }

            Ok(ArchiveInfo {
                entries,
                total_entries,
                total_uncompressed_size,
                total_compressed_size: total_uncompressed_size, // TAR无压缩
                format: ArchiveFormat::Tar,
                has_more: total_entries > limit as u64,
            })
        })
        .await
        .map_err(|e| ArchiveError::IoError(e.to_string()))?
    }

    /// 分析 TAR.GZ 压缩包
    async fn analyze_tar_gz<R>(
        reader: R,
        max_entries: Option<usize>,
    ) -> Result<ArchiveInfo, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        tokio::task::spawn_blocking(move || {
            use tar::Archive;
            use flate2::read::GzDecoder;

            let decoder = GzDecoder::new(reader);
            let mut archive = Archive::new(decoder);
            let mut entries = Vec::new();
            let mut total_entries = 0u64;
            let mut total_uncompressed_size = 0u64;
            let limit = max_entries.unwrap_or(usize::MAX);

            for (index, entry) in archive.entries()
                .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
                .enumerate()
            {
                if index >= limit {
                    break;
                }

                let entry = entry.map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;
                let header = entry.header();

                let path = entry.path()
                    .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
                    .to_string_lossy()
                    .to_string();

                let name = std::path::Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());

                let size = header.size().unwrap_or(0);
                let modified = header.mtime()
                    .ok()
                    .and_then(|timestamp| chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0))
                    .map(|dt| dt.to_rfc3339());

                let entry_info = ArchiveEntry {
                    path,
                    name,
                    size,
                    compressed_size: None, // 压缩大小在TAR.GZ中难以确定
                    modified,
                    is_directory: header.entry_type().is_dir(),
                    is_encrypted: false,
                    crc32: None,
                };

                total_uncompressed_size += size;
                entries.push(entry_info);
                total_entries += 1;
            }

            Ok(ArchiveInfo {
                entries,
                total_entries,
                total_uncompressed_size,
                total_compressed_size: 0, // 无法准确计算TAR.GZ的总压缩大小
                format: ArchiveFormat::TarGz,
                has_more: total_entries > limit as u64,
            })
        })
        .await
        .map_err(|e| ArchiveError::IoError(e.to_string()))?
    }

    /// 从 TAR 中提取文件
    async fn extract_file_from_tar<R>(
        reader: R,
        file_path: &str,
        max_size: Option<u64>,
    ) -> Result<FilePreview, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        let file_path = file_path.to_string();

        tokio::task::spawn_blocking(move || {
            use tar::Archive;

            let mut archive = Archive::new(reader);

            for entry in archive.entries()
                .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
            {
                let mut entry = entry.map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;
                let path = entry.path()
                    .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
                    .to_string_lossy()
                    .to_string();

                if path == file_path {
                    let total_size = entry.header().size().unwrap_or(0);
                    let read_size = max_size.unwrap_or(total_size).min(total_size);

                    let mut buffer = vec![0u8; read_size as usize];
                    let bytes_read = entry.read(&mut buffer)
                        .map_err(|e| ArchiveError::ExtractionFailed(e.to_string()))?;

                    buffer.truncate(bytes_read);

                    return Ok(FilePreview {
                        content: buffer,
                        is_truncated: bytes_read < total_size as usize,
                        total_size,
                        preview_size: bytes_read as u64,
                    });
                }
            }

            Err(ArchiveError::FileNotFound(file_path))
        })
        .await
        .map_err(|e| ArchiveError::IoError(e.to_string()))?
    }

    /// 从 TAR.GZ 中提取文件
    async fn extract_file_from_tar_gz<R>(
        reader: R,
        file_path: &str,
        max_size: Option<u64>,
    ) -> Result<FilePreview, ArchiveError>
    where
        R: Read + Seek + Send + 'static,
    {
        let file_path = file_path.to_string();

        tokio::task::spawn_blocking(move || {
            use tar::Archive;
            use flate2::read::GzDecoder;

            let decoder = GzDecoder::new(reader);
            let mut archive = Archive::new(decoder);

            for entry in archive.entries()
                .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
            {
                let mut entry = entry.map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?;
                let path = entry.path()
                    .map_err(|e| ArchiveError::InvalidArchive(e.to_string()))?
                    .to_string_lossy()
                    .to_string();

                if path == file_path {
                    let total_size = entry.header().size().unwrap_or(0);
                    let read_size = max_size.unwrap_or(total_size).min(total_size);

                    let mut buffer = vec![0u8; read_size as usize];
                    let bytes_read = entry.read(&mut buffer)
                        .map_err(|e| ArchiveError::ExtractionFailed(e.to_string()))?;

                    buffer.truncate(bytes_read);

                    return Ok(FilePreview {
                        content: buffer,
                        is_truncated: bytes_read < total_size as usize,
                        total_size,
                        preview_size: bytes_read as u64,
                    });
                }
            }

            Err(ArchiveError::FileNotFound(file_path))
        })
        .await
        .map_err(|e| ArchiveError::IoError(e.to_string()))?
    }
}