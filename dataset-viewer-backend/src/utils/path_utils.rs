/// 路径工具函数
pub fn normalize_path(path: &str) -> String {
    let mut path = path.trim().to_string();
    
    // 移除重复的斜杠
    while path.contains("//") {
        path = path.replace("//", "/");
    }
    
    // 确保不以斜杠开头（除了根路径）
    if path.starts_with('/') && path.len() > 1 {
        path = path[1..].to_string();
    }
    
    // 移除末尾的斜杠
    if path.ends_with('/') && path.len() > 1 {
        path = path[..path.len() - 1].to_string();
    }
    
    path
}

/// 连接路径
pub fn join_paths(base: &str, path: &str) -> String {
    let base = normalize_path(base);
    let path = normalize_path(path);
    
    if base.is_empty() {
        path
    } else if path.is_empty() {
        base
    } else {
        format!("{}/{}", base, path)
    }
}

/// 获取文件扩展名
pub fn get_file_extension(filename: &str) -> Option<String> {
    std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// 判断是否为压缩文件
pub fn is_archive_file(filename: &str) -> bool {
    match get_file_extension(filename).as_deref() {
        Some("zip") | Some("tar") | Some("gz") | Some("bz2") | Some("xz") 
        | Some("7z") | Some("rar") | Some("tgz") | Some("txz") => true,
        _ => false,
    }
}

/// 推断 MIME 类型
pub fn guess_mime_type(filename: &str) -> String {
    match get_file_extension(filename).as_deref() {
        // 文本文件
        Some("txt") => "text/plain",
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("xml") => "application/xml",
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("ts") => "application/typescript",
        Some("md") => "text/markdown",
        Some("yaml") | Some("yml") => "application/x-yaml",
        Some("toml") => "application/toml",
        
        // 数据文件
        Some("parquet") => "application/x-parquet",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("xls") => "application/vnd.ms-excel",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("doc") => "application/msword",
        Some("pdf") => "application/pdf",
        
        // 压缩文件
        Some("zip") => "application/zip",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        Some("bz2") => "application/x-bzip2",
        Some("xz") => "application/x-xz",
        Some("7z") => "application/x-7z-compressed",
        Some("rar") => "application/vnd.rar",
        
        // 图像文件
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        
        // 音视频文件
        Some("mp4") => "video/mp4",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("wmv") => "video/x-ms-wmv",
        Some("flv") => "video/x-flv",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("flac") => "audio/flac",
        Some("ogg") => "audio/ogg",
        
        // 默认
        _ => "application/octet-stream",
    }.to_string()
}