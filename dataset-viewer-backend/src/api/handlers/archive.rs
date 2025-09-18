use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

use crate::{
    api::types::ApiResponse,
    archive::{ArchiveInfo, FilePreview},
    state::AppState,
    Error,
};

/// 获取压缩包信息请求
#[derive(serde::Deserialize)]
pub struct GetArchiveInfoRequest {
    pub file_path: String,
    pub max_entries: Option<usize>,
}

/// 获取压缩包文件请求
#[derive(serde::Deserialize)]
pub struct GetArchiveFileRequest {
    pub archive_path: String,
    pub file_path: String,
    pub max_size: Option<u64>,
    pub offset: Option<u64>,
}

/// 获取压缩包信息
pub async fn get_archive_info(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<GetArchiveInfoRequest>,
) -> Result<Json<ApiResponse<ArchiveInfo>>, Error> {
    if !state.storage_manager.session_exists(&session_id).await {
        return Err(Error::NotFound("Session not found".to_string()));
    }

    // 1. 从文件路径推断压缩包格式
    let archive_format = {
        let path = std::path::Path::new(&request.file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        crate::archive::ArchiveFormat::from_extension(extension)
    };

    // 检查是否支持该格式
    match archive_format {
        crate::archive::ArchiveFormat::Unknown => {
            return Err(Error::BadRequest("不支持的压缩包格式".to_string()));
        }
        crate::archive::ArchiveFormat::SevenZ
        | crate::archive::ArchiveFormat::Rar
        | crate::archive::ArchiveFormat::TarBz2
        | crate::archive::ArchiveFormat::TarXz
        | crate::archive::ArchiveFormat::Gzip => {
            return Err(Error::BadRequest(format!("暂不支持 {:?} 格式", archive_format)));
        }
        _ => {}
    }

    // 2. 使用流式分析来处理大 ZIP 文件（参考 Tauri 实现）
    const MIN_ZIP_SIZE: u64 = 22; // 最小ZIP文件大小（EOCD记录）
    const MAX_FOOTER_SIZE: u64 = 65536; // 最多读取64KB的文件尾部
    const MAX_ZIP_SIZE: u64 = 500 * 1024 * 1024 * 1024; // 500GB文件大小限制
    const ZIP_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04]; // "PK\x03\x04"
    const EOCD_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06]; // "PK\x05\x06"

    // 获取文件大小
    println!("开始获取文件大小: session_id={}, file_path={}", session_id, request.file_path);
    let file_size = state.storage_manager
        .get_file_size(&session_id, &request.file_path)
        .await
        .map_err(|e| Error::Internal(format!("获取文件大小失败: {}", e)))?;

    println!("文件大小: {} 字节 ({:.2} GB)", file_size, file_size as f64 / (1024.0 * 1024.0 * 1024.0));

    // 检查文件大小是否足够
    if file_size < MIN_ZIP_SIZE {
        return Err(Error::BadRequest(format!(
            "文件太小，不是有效的ZIP文件 ({} 字节 < {} 字节)",
            file_size, MIN_ZIP_SIZE
        )));
    }

    // 检查最大文件大小限制（防止处理过大的文件）
    if file_size > MAX_ZIP_SIZE {
        return Err(Error::BadRequest(format!(
            "ZIP文件太大: {} 字节，超过500GB限制",
            file_size
        )));
    }

    // 首先读取文件头部验证是否为ZIP文件
    println!("开始读取文件头部...");
    let header_data = state.storage_manager
        .read_file_range(&session_id, &request.file_path, 0, 4)
        .await
        .map_err(|e| Error::Internal(format!("读取文件头部失败: {}", e)))?;

    println!("读取到头部数据: {:?}", header_data);

    if header_data.len() < 4 || &header_data[0..4] != &ZIP_SIGNATURE {
        return Err(Error::BadRequest("文件不是有效的ZIP格式".to_string()));
    }

    // 读取文件末尾来查找中央目录（EOCD记录）
    let footer_size = std::cmp::min(MAX_FOOTER_SIZE, file_size);
    let start_pos = file_size.saturating_sub(footer_size);

    let footer_data = state.storage_manager
        .read_file_range(&session_id, &request.file_path, start_pos, footer_size)
        .await
        .map_err(|e| Error::Internal(format!("读取文件尾部失败: {}", e)))?;

    if footer_data.len() != footer_size as usize {
        return Err(Error::Internal(format!(
            "读取数据长度不匹配: 期望 {}, 实际 {}",
            footer_size,
            footer_data.len()
        )));
    }

    // 查找EOCD记录
    let eocd_pos = find_eocd(&footer_data)
        .ok_or_else(|| Error::BadRequest("无法在ZIP文件中找到EOCD记录，文件可能损坏或不是有效的ZIP文件".to_string()))?;

    let eocd_data = &footer_data[eocd_pos..];
    if eocd_data.len() < 22 {
        return Err(Error::BadRequest(format!(
            "EOCD记录长度不足: 只有 {} 字节，需要 22 字节",
            eocd_data.len()
        )));
    }

    // 解析EOCD记录获取中央目录信息
    let total_entries = u16::from_le_bytes([eocd_data[10], eocd_data[11]]) as u64;
    let cd_size = u32::from_le_bytes([eocd_data[12], eocd_data[13], eocd_data[14], eocd_data[15]]) as u64;
    let cd_offset = u32::from_le_bytes([eocd_data[16], eocd_data[17], eocd_data[18], eocd_data[19]]) as u64;

    // 验证条目数量的合理性
    const MAX_ENTRIES: u64 = 1_000_000; // 100万个文件数量限制
    if total_entries > MAX_ENTRIES {
        return Err(Error::BadRequest(format!(
            "ZIP文件中条目过多: {}，超过{}限制",
            total_entries, MAX_ENTRIES
        )));
    }

    if cd_size > file_size {
        return Err(Error::BadRequest(format!(
            "中央目录大小 ({}) 超过文件大小 ({})",
            cd_size, file_size
        )));
    }

    // 验证中央目录大小的合理性
    const MAX_CD_SIZE: u64 = 500 * 1024 * 1024; // 500MB中央目录大小限制
    if cd_size > MAX_CD_SIZE {
        return Err(Error::BadRequest(format!(
            "中央目录太大: {} 字节，超过500MB限制",
            cd_size
        )));
    }

    // 读取中央目录
    let cd_data = state.storage_manager
        .read_file_range(&session_id, &request.file_path, cd_offset, cd_size)
        .await
        .map_err(|e| Error::Internal(format!("读取中央目录失败: {}", e)))?;

    if cd_data.len() != cd_size as usize {
        return Err(Error::Internal(format!(
            "读取中央目录数据长度不匹配: 期望 {}, 实际 {}",
            cd_size,
            cd_data.len()
        )));
    }

    // 3. 解析中央目录获取文件列表
    let archive_info = parse_central_directory(&cd_data, total_entries, request.max_entries)
        .map_err(|e| Error::Internal(format!("解析中央目录失败: {}", e)))?;

    Ok(Json(ApiResponse::success(archive_info)))
}

/// 获取压缩包中的文件内容
pub async fn get_archive_file(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(request): Json<GetArchiveFileRequest>,
) -> Result<Json<ApiResponse<FilePreview>>, Error> {
    if !state.storage_manager.session_exists(&session_id).await {
        return Err(Error::NotFound("Session not found".to_string()));
    }

    // 1. 从文件路径推断压缩包格式
    let archive_format = {
        let path = std::path::Path::new(&request.archive_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        crate::archive::ArchiveFormat::from_extension(extension)
    };

    // 检查是否支持该格式
    match archive_format {
        crate::archive::ArchiveFormat::Unknown => {
            return Err(Error::BadRequest("不支持的压缩包格式".to_string()));
        }
        crate::archive::ArchiveFormat::SevenZ
        | crate::archive::ArchiveFormat::Rar
        | crate::archive::ArchiveFormat::TarBz2
        | crate::archive::ArchiveFormat::TarXz
        | crate::archive::ArchiveFormat::Gzip => {
            return Err(Error::BadRequest(format!("暂不支持 {:?} 格式", archive_format)));
        }
        _ => {}
    }

    // 2. 从存储客户端读取压缩包文件
    let file_data = if let Some(offset) = request.offset {
        // 如果指定了偏移量，使用范围读取来节省内存
        let max_size = request.max_size.unwrap_or(50 * 1024 * 1024); // 默认最大50MB
        state.storage_manager
            .get_file_content(&session_id, &request.archive_path, Some(offset), Some(max_size))
            .await
            .map_err(|e| Error::Internal(format!("读取压缩包文件失败: {}", e)))?
    } else {
        // 读取完整的压缩包文件
        state.storage_manager
            .get_file_content(&session_id, &request.archive_path, None, None)
            .await
            .map_err(|e| Error::Internal(format!("读取压缩包文件失败: {}", e)))?
    };

    // 3. 使用 ArchiveHandler 提取指定文件
    let cursor = std::io::Cursor::new(file_data.content);
    let file_preview = crate::archive::ArchiveHandler::extract_file(
        cursor,
        archive_format,
        &request.file_path,
        request.max_size,
    )
    .await
    .map_err(|e| Error::Internal(format!("提取文件失败: {}", e)))?;

    Ok(Json(ApiResponse::success(file_preview)))
}

/// 在数据中查找EOCD记录位置
fn find_eocd(data: &[u8]) -> Option<usize> {
    const EOCD_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    const MIN_EOCD_SIZE: usize = 22;

    if data.len() < MIN_EOCD_SIZE {
        return None;
    }

    // 从后往前搜索EOCD签名，优化搜索性能
    for i in (0..=data.len() - MIN_EOCD_SIZE).rev() {
        if data[i..i + 4] == EOCD_SIGNATURE {
            // 验证这是一个有效的EOCD记录
            let comment_len = u16::from_le_bytes([data[i + 20], data[i + 21]]) as usize;
            if i + MIN_EOCD_SIZE + comment_len == data.len() {
                return Some(i);
            }
        }
    }

    None
}

/// 解析中央目录获取文件列表
fn parse_central_directory(
    cd_data: &[u8],
    total_entries: u64,
    max_entries: Option<usize>,
) -> Result<crate::archive::ArchiveInfo, String> {
    use crate::archive::{ArchiveEntry, ArchiveInfo};
    use std::collections::HashMap;

    const CD_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];

    let mut entries = Vec::new();
    let mut offset = 0usize;
    let mut entries_parsed = 0u64;
    let max_parse = max_entries.unwrap_or(10000).min(total_entries as usize);

    while offset < cd_data.len() && entries_parsed < max_parse as u64 {
        // 检查是否有足够的数据读取中央目录头
        if offset + 46 > cd_data.len() {
            break;
        }

        // 验证中央目录签名
        if &cd_data[offset..offset + 4] != &CD_SIGNATURE {
            return Err(format!("无效的中央目录签名，偏移量: {}", offset));
        }

        // 解析中央目录头
        let filename_len = u16::from_le_bytes([cd_data[offset + 28], cd_data[offset + 29]]) as usize;
        let extra_len = u16::from_le_bytes([cd_data[offset + 30], cd_data[offset + 31]]) as usize;
        let comment_len = u16::from_le_bytes([cd_data[offset + 32], cd_data[offset + 33]]) as usize;

        // 检查是否有足够的数据读取文件名
        if offset + 46 + filename_len + extra_len + comment_len > cd_data.len() {
            break;
        }

        // 读取文件名
        let filename_data = &cd_data[offset + 46..offset + 46 + filename_len];
        let filename = String::from_utf8_lossy(filename_data).to_string();

        // 解析文件属性
        let uncompressed_size = u32::from_le_bytes([
            cd_data[offset + 24], cd_data[offset + 25],
            cd_data[offset + 26], cd_data[offset + 27]
        ]) as u64;

        let compressed_size = u32::from_le_bytes([
            cd_data[offset + 20], cd_data[offset + 21],
            cd_data[offset + 22], cd_data[offset + 23]
        ]) as u64;

        let crc32 = u32::from_le_bytes([
            cd_data[offset + 16], cd_data[offset + 17],
            cd_data[offset + 18], cd_data[offset + 19]
        ]);

        let external_attr = u32::from_le_bytes([
            cd_data[offset + 38], cd_data[offset + 39],
            cd_data[offset + 40], cd_data[offset + 41]
        ]);

        // 判断是否为目录（检查文件名是否以 '/' 结尾或者外部属性）
        let is_dir = filename.ends_with('/') || (external_attr & 0x10) != 0;

        // 获取最后修改时间
        let mod_time = u16::from_le_bytes([cd_data[offset + 12], cd_data[offset + 13]]);
        let mod_date = u16::from_le_bytes([cd_data[offset + 14], cd_data[offset + 15]]);

        // DOS时间转换为时间戳（简化版本）
        let timestamp = dos_time_to_timestamp(mod_date, mod_time);

        entries.push(ArchiveEntry {
            path: filename.clone(),
            name: filename,
            size: if is_dir { 0 } else { uncompressed_size },
            compressed_size: Some(compressed_size),
            modified: Some(timestamp.to_string()),
            is_directory: is_dir,
            is_encrypted: false,
            crc32: Some(crc32),
        });

        // 移动到下一个条目
        offset += 46 + filename_len + extra_len + comment_len;
        entries_parsed += 1;
    }

    let has_more = entries_parsed < total_entries;

    Ok(ArchiveInfo {
        entries,
        total_entries: total_entries as u64,
        total_uncompressed_size: 0, // 我们没有计算总的未压缩大小
        total_compressed_size: 0,   // 我们没有计算总的压缩大小
        format: crate::archive::ArchiveFormat::Zip,
        has_more,
    })
}

/// 将DOS时间转换为时间戳
fn dos_time_to_timestamp(date: u16, time: u16) -> i64 {
    // DOS时间格式转换（简化版本）
    let year = ((date >> 9) & 0x7f) + 1980;
    let month = (date >> 5) & 0x0f;
    let day = date & 0x1f;

    let hour = (time >> 11) & 0x1f;
    let minute = (time >> 5) & 0x3f;
    let second = (time & 0x1f) * 2;

    // 创建一个简单的时间戳（这是一个简化的实现）
    // 在生产环境中，您可能想要使用更精确的时间库
    let base_timestamp = 315532800i64; // 1980-01-01 00:00:00 UTC
    let days_since_1980 = (year as i64 - 1980) * 365 + (month as i64 - 1) * 30 + day as i64;

    base_timestamp + days_since_1980 * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64
}