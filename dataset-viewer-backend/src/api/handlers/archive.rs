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
    println!("=== get_archive_file API 调用开始 ===");
    println!("Session ID: {}", session_id);
    println!("请求: archive_path='{}', file_path='{}', max_size={:?}, offset={:?}",
        request.archive_path, request.file_path, request.max_size, request.offset);

    if !state.storage_manager.session_exists(&session_id).await {
        println!("错误: Session {} 不存在", session_id);
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

    println!("开始读取压缩包文件: {}", request.archive_path);

    // 2. 从存储客户端读取压缩包文件
    let file_data = if let Some(offset) = request.offset {
        println!("使用偏移量读取: offset={}, max_size={:?}", offset, request.max_size);
        // 如果指定了偏移量，使用范围读取来节省内存
        let max_size = request.max_size.unwrap_or(50 * 1024 * 1024); // 默认最大50MB
        state.storage_manager
            .get_file_content(&session_id, &request.archive_path, Some(offset), Some(max_size))
            .await
            .map_err(|e| {
                println!("范围读取压缩包文件失败: {}", e);
                Error::Internal(format!("读取压缩包文件失败: {}", e))
            })?
    } else {
        println!("开始智能ZIP文件处理流程");

        // 智能方案：先获取ZIP文件大小，然后读取末尾部分查找EOCD
        let zip_file_size = state.storage_manager
            .get_file_size(&session_id, &request.archive_path)
            .await
            .map_err(|e| Error::Internal(format!("获取ZIP文件大小失败: {}", e)))?;

        println!("ZIP文件总大小: {} 字节 ({:.2} MB)", zip_file_size, zip_file_size as f64 / 1024.0 / 1024.0);

        // 限制处理的ZIP文件大小
        const MAX_ZIP_SIZE: u64 = 2 * 1024 * 1024 * 1024; // 2GB限制
        if zip_file_size > MAX_ZIP_SIZE {
            return Err(Error::BadRequest(
                "ZIP文件太大（超过2GB），暂不支持预览。".to_string()
            ));
        }

        // 读取ZIP文件末尾部分来查找EOCD记录
        const MAX_EOCD_SEARCH_SIZE: u64 = 64 * 1024; // 搜索末尾64KB
        let eocd_search_size = std::cmp::min(MAX_EOCD_SEARCH_SIZE, zip_file_size);
        let eocd_start = zip_file_size.saturating_sub(eocd_search_size);

        println!("读取ZIP文件末尾用于查找EOCD: 从位置{} 读取{}字节", eocd_start, eocd_search_size);

        let eocd_data = state.storage_manager
            .read_file_range(&session_id, &request.archive_path, eocd_start, eocd_search_size)
            .await
            .map_err(|e| Error::Internal(format!("读取ZIP文件末尾失败: {}", e)))?;

        println!("成功读取ZIP文件末尾，大小: {} 字节", eocd_data.len());

        // 实现流式ZIP处理：解析EOCD记录找到中央目录
        let (cd_offset, cd_size) = parse_eocd_record(&eocd_data, zip_file_size)
            .map_err(|e| Error::BadRequest(format!("ZIP文件格式错误: {}", e)))?;

        println!("找到中央目录: 偏移={}, 大小={} 字节", cd_offset, cd_size);

        // 限制中央目录大小，避免读取过大的目录
        const MAX_CD_SIZE: u64 = 50 * 1024 * 1024; // 50MB
        if cd_size > MAX_CD_SIZE {
            return Err(Error::BadRequest(
                "ZIP文件的中央目录太大，暂不支持处理".to_string()
            ));
        }

        // 读取中央目录
        println!("读取中央目录: 从位置{} 读取{}字节", cd_offset, cd_size);
        let cd_data = state.storage_manager
            .read_file_range(&session_id, &request.archive_path, cd_offset, cd_size)
            .await
            .map_err(|e| Error::Internal(format!("读取中央目录失败: {}", e)))?;

        println!("成功读取中央目录，大小: {} 字节", cd_data.len());

        // 在中央目录中查找目标文件
        let file_entry = find_file_in_central_directory(&cd_data, &request.file_path)
            .map_err(|e| Error::NotFound(format!("文件未找到: {}", e)))?;

        println!("找到目标文件: {} (偏移={}, 压缩大小={}, 原始大小={})",
            file_entry.filename, file_entry.local_header_offset,
            file_entry.compressed_size, file_entry.uncompressed_size);

        // 限制要提取的文件大小
        const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
        if file_entry.uncompressed_size > MAX_FILE_SIZE {
            return Err(Error::BadRequest(format!(
                "目标文件太大 ({:.2} MB)，超过100MB限制",
                file_entry.uncompressed_size as f64 / 1024.0 / 1024.0
            )));
        }

        // 读取文件的本地头部和数据
        let file_data = extract_file_from_zip(
            &state.storage_manager,
            &session_id,
            &request.archive_path,
            &file_entry,
            request.max_size
        ).await?;

        // 构造文件内容结构
        crate::storage::FileContent {
            content: file_data,
            mime_type: None,
            encoding: None,
            size: file_entry.uncompressed_size,
        }
    };

    println!("成功读取压缩包文件，大小: {} 字节", file_data.content.len());

    // 3. 根据压缩包格式处理文件提取
    let file_preview = if matches!(archive_format, crate::archive::ArchiveFormat::Zip) {
        // 对于ZIP文件，我们已经通过流式处理解压了数据，直接返回
        println!("ZIP文件已通过流式处理解压完成，直接返回数据");
        let content_len = file_data.content.len() as u64;
        crate::archive::FilePreview {
            content: file_data.content,
            is_truncated: false, // TODO: 根据max_size判断是否截断
            total_size: file_data.size,
            preview_size: content_len,
        }
    } else {
        // 对于其他格式，使用 ArchiveHandler
        println!("开始提取文件: {}", request.file_path);
        let cursor = std::io::Cursor::new(file_data.content);
        crate::archive::ArchiveHandler::extract_file(
            cursor,
            archive_format,
            &request.file_path,
            request.max_size,
        )
        .await
        .map_err(|e| {
            println!("提取文件失败: {}", e);
            Error::Internal(format!("提取文件失败: {}", e))
        })?
    };

    println!("成功提取文件，预览大小: {} 字节", file_preview.preview_size);

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

/// ZIP文件中的文件条目信息
#[derive(Debug)]
struct ZipFileEntry {
    filename: String,
    compressed_size: u64,
    uncompressed_size: u64,
    local_header_offset: u64,
    compression_method: u16,
}

/// 解析EOCD记录，返回中央目录的偏移和大小
fn parse_eocd_record(eocd_data: &[u8], _file_size: u64) -> Result<(u64, u64), String> {
    const EOCD_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06]; // "PK\x05\x06"
    const MIN_EOCD_SIZE: usize = 22;

    if eocd_data.len() < MIN_EOCD_SIZE {
        return Err("数据太短，无法包含EOCD记录".to_string());
    }

    // 从后往前搜索EOCD签名
    for i in (0..=eocd_data.len() - MIN_EOCD_SIZE).rev() {
        if eocd_data[i..i + 4] == EOCD_SIGNATURE {
            let comment_len = u16::from_le_bytes([eocd_data[i + 20], eocd_data[i + 21]]) as usize;
            if i + MIN_EOCD_SIZE + comment_len == eocd_data.len() {
                // 找到有效的EOCD记录
                let cd_size = u32::from_le_bytes([
                    eocd_data[i + 12], eocd_data[i + 13],
                    eocd_data[i + 14], eocd_data[i + 15]
                ]) as u64;
                let cd_offset = u32::from_le_bytes([
                    eocd_data[i + 16], eocd_data[i + 17],
                    eocd_data[i + 18], eocd_data[i + 19]
                ]) as u64;

                return Ok((cd_offset, cd_size));
            }
        }
    }

    Err("未找到有效的EOCD记录".to_string())
}

/// 在中央目录中查找指定文件
fn find_file_in_central_directory(cd_data: &[u8], target_filename: &str) -> Result<ZipFileEntry, String> {
    const CD_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02]; // "PK\x01\x02"

    let mut offset = 0;

    while offset + 46 <= cd_data.len() {
        // 检查中央目录签名
        if &cd_data[offset..offset + 4] != &CD_SIGNATURE {
            break;
        }

        // 读取文件名长度
        let filename_len = u16::from_le_bytes([cd_data[offset + 28], cd_data[offset + 29]]) as usize;
        let extra_len = u16::from_le_bytes([cd_data[offset + 30], cd_data[offset + 31]]) as usize;
        let comment_len = u16::from_le_bytes([cd_data[offset + 32], cd_data[offset + 33]]) as usize;

        if offset + 46 + filename_len + extra_len + comment_len > cd_data.len() {
            break;
        }

        // 读取文件名
        let filename_bytes = &cd_data[offset + 46..offset + 46 + filename_len];
        let filename = String::from_utf8_lossy(filename_bytes);

        // 检查是否匹配目标文件
        if filename == target_filename {
            // 读取文件信息
            let compression_method = u16::from_le_bytes([cd_data[offset + 10], cd_data[offset + 11]]);
            let compressed_size = u32::from_le_bytes([
                cd_data[offset + 20], cd_data[offset + 21],
                cd_data[offset + 22], cd_data[offset + 23]
            ]) as u64;
            let uncompressed_size = u32::from_le_bytes([
                cd_data[offset + 24], cd_data[offset + 25],
                cd_data[offset + 26], cd_data[offset + 27]
            ]) as u64;
            let local_header_offset = u32::from_le_bytes([
                cd_data[offset + 42], cd_data[offset + 43],
                cd_data[offset + 44], cd_data[offset + 45]
            ]) as u64;

            return Ok(ZipFileEntry {
                filename: filename.to_string(),
                compressed_size,
                uncompressed_size,
                local_header_offset,
                compression_method,
            });
        }

        // 移动到下一个条目
        offset += 46 + filename_len + extra_len + comment_len;
    }

    Err(format!("文件 '{}' 未在中央目录中找到", target_filename))
}

/// 从ZIP文件中提取特定文件的数据
async fn extract_file_from_zip(
    storage_manager: &crate::storage::StorageManager,
    session_id: &str,
    archive_path: &str,
    file_entry: &ZipFileEntry,
    max_size: Option<u64>,
) -> Result<Vec<u8>, crate::Error> {
    // 首先读取本地文件头部以获取完整的偏移信息
    const LOCAL_HEADER_SIZE: u64 = 30;
    let local_header = storage_manager
        .read_file_range(session_id, archive_path, file_entry.local_header_offset, LOCAL_HEADER_SIZE)
        .await
        .map_err(|e| crate::Error::Internal(format!("读取本地头部失败: {}", e)))?;

    if local_header.len() < 30 {
        return Err(crate::Error::Internal("本地头部数据不完整".to_string()));
    }

    // 解析本地头部获取文件名和额外字段的长度
    let filename_len = u16::from_le_bytes([local_header[26], local_header[27]]) as u64;
    let extra_len = u16::from_le_bytes([local_header[28], local_header[29]]) as u64;

    // 计算实际数据的开始位置
    let data_offset = file_entry.local_header_offset + LOCAL_HEADER_SIZE + filename_len + extra_len;

    // 确定要读取的大小
    let read_size = match max_size {
        Some(max) => std::cmp::min(max, file_entry.compressed_size),
        None => file_entry.compressed_size,
    };

    println!("读取文件数据: 偏移={}, 大小={} 字节", data_offset, read_size);

    // 读取压缩的文件数据
    let compressed_data = storage_manager
        .read_file_range(session_id, archive_path, data_offset, read_size)
        .await
        .map_err(|e| crate::Error::Internal(format!("读取文件数据失败: {}", e)))?;

    // 根据压缩方法解压数据
    match file_entry.compression_method {
        0 => {
            // 无压缩，直接返回
            println!("文件未压缩，直接返回 {} 字节", compressed_data.len());
            Ok(compressed_data)
        }
        8 => {
            // Deflate压缩，需要解压
            println!("使用Deflate解压，压缩数据大小: {} 字节", compressed_data.len());

            use flate2::read::DeflateDecoder;
            use std::io::Read;

            let mut decoder = DeflateDecoder::new(&compressed_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| crate::Error::Internal(format!("解压失败: {}", e)))?;

            println!("解压完成，原始数据大小: {} 字节", decompressed.len());
            Ok(decompressed)
        }
        _ => {
            Err(crate::Error::BadRequest(format!(
                "不支持的压缩方法: {}", file_entry.compression_method
            )))
        }
    }
}