use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use sha1::Digest;

use crate::storage::{
    ConnectionConfig, DirectoryResult, FileInfo, ListOptions, StorageClient, StorageError,
};

/// OSS/S3 客户端
pub struct OSSClient {
    client: Client,
    config: ConnectionConfig,
    connected: bool,
    endpoint: String,
    access_key: String,
    secret_key: String,
    bucket: String,
    region: Option<String>,
}

impl OSSClient {
    pub fn new(config: ConnectionConfig) -> Result<Self, StorageError> {
        let endpoint = config
            .endpoint
            .clone()
            .or_else(|| config.url.clone())
            .ok_or_else(|| StorageError::InvalidConfig("OSS endpoint is required".to_string()))?;

        let access_key = config
            .access_key
            .clone()
            .ok_or_else(|| StorageError::InvalidConfig("OSS access key is required".to_string()))?;

        let secret_key = config
            .secret_key
            .clone()
            .ok_or_else(|| StorageError::InvalidConfig("OSS secret key is required".to_string()))?;

        let bucket = config
            .bucket
            .clone()
            .ok_or_else(|| StorageError::InvalidConfig("OSS bucket is required".to_string()))?;

        let region = config.region.clone();

        // 创建带超时配置的客户端，适合大文件传输
        let client = Client::builder()
            .timeout(Duration::from_secs(300))  // 5分钟总超时
            .connect_timeout(Duration::from_secs(30))  // 30秒连接超时
            .read_timeout(Duration::from_secs(120))  // 2分钟读取超时
            .build()
            .map_err(|e| StorageError::InvalidConfig(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            connected: false,
            endpoint,
            access_key,
            secret_key,
            bucket,
            region,
        })
    }

    /// 构建对象URL - 腾讯云COS格式
    fn build_object_url(&self, object_path: &str) -> String {
        // 腾讯云 COS URL 格式: https://{bucket}.cos.{region}.myqcloud.com/{object}
        // 但我们需要处理 endpoint 可能已经包含存储桶名的情况
        if self.endpoint.contains(&self.bucket) {
            // endpoint 已经包含存储桶，直接使用
            format!("{}/{}", self.endpoint, urlencoding::encode(object_path))
        } else {
            // 需要构建完整的 COS URL
            // 从 endpoint 中提取域名部分
            let domain = self.endpoint
                .trim_start_matches("https://")
                .trim_start_matches("http://");

            format!("https://{}.{}/{}", self.bucket, domain, urlencoding::encode(object_path))
        }
    }

    /// 构建签名头部 - 腾讯云COS签名算法
    fn build_auth_headers(&self, method: &str, path: &str) -> HashMap<String, String> {
        use chrono::Utc;
        use hmac::{Hmac, Mac};
        use sha1::{Sha1, Digest};
        use std::collections::BTreeMap;

        type HmacSha1 = Hmac<Sha1>;

        let mut headers = HashMap::new();

        // 1. 生成 KeyTime
        let now = Utc::now().timestamp();
        let expire_time = now + 3600; // 1小时后过期
        let key_time = format!("{};{}", now, expire_time);

        // 2. 生成 SignKey
        let mut sign_key_mac = HmacSha1::new_from_slice(self.secret_key.as_bytes()).unwrap();
        sign_key_mac.update(key_time.as_bytes());
        let sign_key = hex::encode(sign_key_mac.finalize().into_bytes());

        // 3. 生成 UrlParamList 和 HeaderList
        let url_param_list = ""; // 对于简单的GET请求，暂时为空
        let header_list = "host"; // 包含host头部

        // 4. 生成 HttpString
        // 需要确保 host 匹配实际请求的 URL
        let host = if self.endpoint.contains(&self.bucket) {
            self.endpoint.replace("https://", "").replace("http://", "")
        } else {
            let domain = self.endpoint
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            format!("{}.{}", self.bucket, domain)
        };
        let http_string = format!(
            "{}\n{}\n{}\n{}\n",
            method.to_lowercase(),
            path,
            url_param_list,
            format!("host={}", host)
        );

        // 5. 生成 StringToSign
        let http_string_sha1 = hex::encode(sha1::Sha1::digest(http_string.as_bytes()));
        let string_to_sign = format!(
            "sha1\n{}\n{}\n",
            key_time,
            http_string_sha1
        );

        // 6. 生成 Signature
        let mut signature_mac = HmacSha1::new_from_slice(sign_key.as_bytes()).unwrap();
        signature_mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(signature_mac.finalize().into_bytes());

        // 7. 生成 Authorization
        let authorization = format!(
            "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
            self.access_key,
            key_time,
            key_time,
            header_list,
            url_param_list,
            signature
        );

        // 添加必要的头部
        headers.insert("Host".to_string(), host);
        headers.insert("Authorization".to_string(), authorization);

        headers
    }

    /// 构建签名头部 - 带URL参数版本
    fn build_auth_headers_with_params(&self, method: &str, path: &str, params: &[(String, String)]) -> HashMap<String, String> {
        use chrono::Utc;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use std::collections::BTreeMap;

        type HmacSha1 = Hmac<Sha1>;

        let mut headers = HashMap::new();

        // 1. 生成 KeyTime
        let now = Utc::now().timestamp();
        let expire_time = now + 3600; // 1小时后过期
        let key_time = format!("{};{}", now, expire_time);

        // 2. 生成 SignKey
        let mut sign_key_mac = HmacSha1::new_from_slice(self.secret_key.as_bytes()).unwrap();
        sign_key_mac.update(key_time.as_bytes());
        let sign_key = hex::encode(sign_key_mac.finalize().into_bytes());

        // 3. 生成 UrlParamList 和 HeaderList
        // 注意：腾讯云COS要求参数名和值都需要URL编码，但在UrlParamList中使用原始参数名
        let mut sorted_params_for_signature: BTreeMap<String, String> = BTreeMap::new();
        let mut param_names_for_list: Vec<String> = Vec::new();

        for (key, value) in params {
            // 对于签名字符串，需要URL编码
            let encoded_key = urlencoding::encode(key).to_lowercase();
            let encoded_value = urlencoding::encode(value);
            sorted_params_for_signature.insert(encoded_key.clone(), encoded_value.to_string());

            // 对于UrlParamList，使用小写的原始参数名
            param_names_for_list.push(key.to_lowercase());
        }

        param_names_for_list.sort();
        let url_param_list = param_names_for_list.join(";");
        let header_list = "host"; // 包含host头部

        // 4. 生成 HttpString
        let bucket_url = if self.endpoint.contains(&self.bucket) {
            self.endpoint.clone()
        } else {
            format!("https://{}.cos.ap-chengdu.myqcloud.com", self.bucket)
        };
        let host = bucket_url.replace("https://", "").replace("http://", "");
        let url_param_string = sorted_params_for_signature.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let http_string = format!(
            "{}\n{}\n{}\nhost={}\n",
            method.to_lowercase(),
            path,
            url_param_string,
            host
        );

        // 5. 生成 StringToSign
        let http_string_sha1 = hex::encode(sha1::Sha1::digest(http_string.as_bytes()));
        let string_to_sign = format!(
            "sha1\n{}\n{}\n",
            key_time,
            http_string_sha1
        );

        // 6. 生成 Signature
        let mut signature_mac = HmacSha1::new_from_slice(sign_key.as_bytes()).unwrap();
        signature_mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(signature_mac.finalize().into_bytes());

        // 7. 生成 Authorization
        let authorization = format!(
            "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
            self.access_key,
            key_time,
            key_time,
            header_list,
            url_param_list,
            signature
        );

        // 添加必要的头部
        headers.insert("Host".to_string(), host);
        headers.insert("Authorization".to_string(), authorization);

        headers
    }

    /// 解析ListObjects响应
    fn parse_list_objects_response(&self, xml: &str, path: &str) -> Result<DirectoryResult, StorageError> {
        use std::collections::HashMap;

        let mut files = Vec::new();
        let mut has_more = false;
        let mut next_marker = None;

        // 简单的XML解析 - 查找Contents和CommonPrefixes元素
        let lines: Vec<&str> = xml.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // 解析文件对象
            if line.starts_with("<Contents>") {
                if let Some(file_info) = self.parse_object_from_xml(&lines, &mut i) {
                    files.push(file_info);
                }
            }
            // 解析目录前缀
            else if line.starts_with("<CommonPrefixes>") {
                if let Some(dir_info) = self.parse_prefix_from_xml(&lines, &mut i) {
                    files.push(dir_info);
                }
            }
            // 检查是否有更多结果
            else if line.contains("<IsTruncated>true</IsTruncated>") {
                has_more = true;
            }
            // 获取下一页标记
            else if line.starts_with("<NextContinuationToken>") {
                let end_tag = "</NextContinuationToken>";
                if let Some(start) = line.find('>') {
                    if let Some(end) = line.find(end_tag) {
                        next_marker = Some(line[start + 1..end].to_string());
                    }
                }
            }

            i += 1;
        }

        Ok(DirectoryResult {
            files,
            has_more,
            next_marker,
            total_count: None,
            path: path.to_string(),
        })
    }

    fn parse_object_from_xml(&self, lines: &[&str], index: &mut usize) -> Option<FileInfo> {
        let mut key = String::new();
        let mut last_modified = String::new();
        let mut size = String::new();
        let mut etag = String::new();

        while *index < lines.len() {
            let line = lines[*index].trim();

            if line.starts_with("<Key>") && line.ends_with("</Key>") {
                key = line.replace("<Key>", "").replace("</Key>", "");
            } else if line.starts_with("<LastModified>") && line.ends_with("</LastModified>") {
                last_modified = line.replace("<LastModified>", "").replace("</LastModified>", "");
            } else if line.starts_with("<Size>") && line.ends_with("</Size>") {
                size = line.replace("<Size>", "").replace("</Size>", "");
            } else if line.starts_with("<ETag>") && line.ends_with("</ETag>") {
                etag = line.replace("<ETag>", "").replace("</ETag>", "").replace("\"", "");
            } else if line == "</Contents>" {
                break;
            }

            *index += 1;
        }

        if !key.is_empty() {
            let filename = if let Some(pos) = key.rfind('/') {
                key[pos + 1..].to_string()
            } else {
                key.clone()
            };

            Some(FileInfo {
                filename: filename.clone(),
                basename: filename,
                lastmod: last_modified,
                size,
                file_type: "file".to_string(),
                mime: None,
                etag: if etag.is_empty() { None } else { Some(etag) },
            })
        } else {
            None
        }
    }

    fn parse_prefix_from_xml(&self, lines: &[&str], index: &mut usize) -> Option<FileInfo> {
        let mut prefix = String::new();

        while *index < lines.len() {
            let line = lines[*index].trim();

            if line.starts_with("<Prefix>") && line.ends_with("</Prefix>") {
                prefix = line.replace("<Prefix>", "").replace("</Prefix>", "");
            } else if line == "</CommonPrefixes>" {
                break;
            }

            *index += 1;
        }

        if !prefix.is_empty() && prefix.ends_with('/') {
            let dirname = prefix.trim_end_matches('/');
            let dirname = if let Some(pos) = dirname.rfind('/') {
                dirname[pos + 1..].to_string()
            } else {
                dirname.to_string()
            };

            Some(FileInfo {
                filename: dirname.clone(),
                basename: dirname,
                lastmod: "".to_string(),
                size: "0".to_string(),
                file_type: "directory".to_string(),
                mime: None,
                etag: None,
            })
        } else {
            None
        }
    }
}

#[async_trait]
impl StorageClient for OSSClient {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<(), StorageError> {
        self.validate_config(config)?;
        
        // TODO: 实现连接测试
        // 例如：列出 bucket 根目录来测试连接
        
        self.connected = true;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<(), StorageError> {
        self.connected = false;
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

        // 构建请求URL - 腾讯云COS格式
        let bucket_url = if self.endpoint.contains(&self.bucket) {
            // 如果endpoint已经包含bucket名称，直接使用
            self.endpoint.clone()
        } else {
            // 构建正确的腾讯云COS URL格式
            format!("https://{}.cos.ap-chengdu.myqcloud.com", self.bucket)
        };
        let url = format!("{}/", bucket_url);
        let prefix = if path.is_empty() { "" } else { path };

        // 构建查询参数
        let mut params = vec![
            ("list-type", "2".to_string()),
            ("delimiter", "/".to_string()),
        ];

        if !prefix.is_empty() {
            params.push(("prefix", prefix.to_string()));
        }

        if let Some(opts) = options {
            if let Some(page_size) = opts.page_size {
                params.push(("max-keys", page_size.to_string()));
            }
            if let Some(marker) = &opts.marker {
                params.push(("continuation-token", marker.clone()));
            }
        }

        // 转换参数类型用于签名
        let string_params: Vec<(String, String)> = params.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let auth_headers = self.build_auth_headers_with_params("GET", "/", &string_params);

        // 发送请求
        let mut request_builder = self.client
            .get(&url)
            .query(&params);

        // 添加认证头部
        for (key, value) in auth_headers {
            request_builder = request_builder.header(&key, &value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| StorageError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(StorageError::RequestFailed(format!(
                "HTTP {}: {}",
                status,
                text
            )));
        }

        let xml_text = response.text().await
            .map_err(|e| StorageError::NetworkError(e.to_string()))?;

        // 添加调试日志
        println!("腾讯云COS API响应: {}", xml_text);

        // 解析XML响应
        self.parse_list_objects_response(&xml_text, path)
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

        // 构建请求 URL
        let object_path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        let url = self.build_object_url(object_path);

        println!("腾讯云COS read_file_range: url={}, path={}, start={}, length={}", url, object_path, start, length);

        // 生成签名头部
        let mut headers = self.build_auth_headers("GET", &format!("/{}", object_path));

        // 添加 Range 头部
        let end = start + length - 1;
        headers.insert("Range".to_string(), format!("bytes={}-{}", start, end));

        println!("腾讯云COS read_file_range: Range头部: bytes={}-{}", start, end);
        println!("腾讯云COS read_file_range: 生成的认证头部: {:?}", headers);

        // 构建请求
        let mut request = self.client.get(&url);
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        println!("腾讯云COS read_file_range: 发送GET请求...");

        // 发送请求
        let response = request.send().await
            .map_err(|e| {
                println!("腾讯云COS read_file_range: 请求失败: {}", e);
                StorageError::RequestFailed(format!("Failed to send GET request: {}", e))
            })?;

        println!("腾讯云COS read_file_range: 收到响应状态: {}", response.status());

        // 对于范围请求，期望 206 Partial Content 状态码
        if !response.status().is_success() && response.status() != 206 {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            println!("腾讯云COS read_file_range: 错误响应 - 状态: {}, 响应体: {}", status, body);
            return Err(StorageError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        println!("腾讯云COS read_file_range: 开始读取响应体...");

        // 获取响应体
        let bytes = response.bytes().await
            .map_err(|e| {
                println!("腾讯云COS read_file_range: 读取响应体失败: {}", e);
                StorageError::RequestFailed(format!("Failed to read response body: {}", e))
            })?;

        println!("腾讯云COS read_file_range: 成功读取 {} 字节", bytes.len());
        Ok(bytes.to_vec())
    }

    async fn read_full_file(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        // 构建请求 URL
        let object_path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        let url = self.build_object_url(object_path);

        // 生成签名头部
        let headers = self.build_auth_headers("GET", &format!("/{}", object_path));

        // 构建请求
        let mut request = self.client.get(&url);
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        // 发送请求
        let response = request.send().await
            .map_err(|e| StorageError::RequestFailed(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(StorageError::RequestFailed(format!("HTTP {}: {}", status, body)));
        }

        // 获取响应体
        let bytes = response.bytes().await
            .map_err(|e| StorageError::RequestFailed(format!("Failed to read response body: {}", e)))?;

        Ok(bytes.to_vec())
    }

    async fn get_file_size(&self, path: &str) -> Result<u64, StorageError> {
        if !self.connected {
            return Err(StorageError::NotConnected);
        }

        // 构建请求 URL
        let object_path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        let url = self.build_object_url(object_path);

        println!("腾讯云COS get_file_size: url={}, path={}", url, object_path);

        // 生成签名头部
        let headers = self.build_auth_headers("HEAD", &format!("/{}", object_path));

        println!("腾讯云COS get_file_size: 生成的认证头部: {:?}", headers);

        // 构建请求
        let mut request = self.client.head(&url);
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        println!("腾讯云COS get_file_size: 发送HEAD请求...");

        // 发送请求
        let response = request.send().await
            .map_err(|e| {
                println!("腾讯云COS get_file_size: 请求失败: {}", e);
                StorageError::RequestFailed(format!("Failed to send HEAD request: {}", e))
            })?;

        println!("腾讯云COS get_file_size: 收到响应状态: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let headers_debug = format!("{:?}", response.headers());
            println!("腾讯云COS get_file_size: 错误响应 - 状态: {}, 头部: {}", status, headers_debug);
            return Err(StorageError::RequestFailed(format!("HTTP {} - Headers: {}", status, headers_debug)));
        }

        // 获取 Content-Length 头部
        let content_length = response.headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| {
                let headers_debug = format!("{:?}", response.headers());
                println!("腾讯云COS get_file_size: 缺少Content-Length头部, 所有头部: {}", headers_debug);
                StorageError::RequestFailed(format!("Missing or invalid Content-Length header. Headers: {}", headers_debug))
            })?;

        println!("腾讯云COS get_file_size: 文件大小: {} 字节", content_length);
        Ok(content_length)
    }

    fn protocol(&self) -> &str {
        "oss"
    }

    fn validate_config(&self, config: &ConnectionConfig) -> Result<(), StorageError> {
        if config.protocol != "oss" && config.protocol != "s3" {
            return Err(StorageError::ProtocolNotSupported(config.protocol.clone()));
        }

        if config.access_key.is_none() {
            return Err(StorageError::InvalidConfig("Access key is required".to_string()));
        }

        if config.secret_key.is_none() {
            return Err(StorageError::InvalidConfig("Secret key is required".to_string()));
        }

        if config.bucket.is_none() {
            return Err(StorageError::InvalidConfig("Bucket is required".to_string()));
        }

        Ok(())
    }

    fn build_protocol_url(&self, path: &str) -> String {
        let clean_path = path.trim_start_matches('/');
        if clean_path.is_empty() {
            format!("{}://{}", self.protocol(), self.bucket)
        } else {
            format!("{}://{}/{}", self.protocol(), self.bucket, clean_path)
        }
    }
}