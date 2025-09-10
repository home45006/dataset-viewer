# Dataset Viewer Backend API

## 基本信息

- **基础 URL**: `http://localhost:8080/api`
- **内容类型**: `application/json`
- **WebSocket**: `ws://localhost:8080/ws`

## 健康检查

### GET /health
检查服务器是否运行正常

**响应**:
```
OK
```

### GET /status
获取服务器详细状态

**响应**:
```json
{
  "status": "success",
  "data": {
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "active_connections": 5,
    "active_sessions": 3,
    "memory_usage": 1024000
  }
}
```

## 存储连接管理

### POST /storage/connect
连接到存储服务

**请求体**:
```json
{
  "config": {
    "protocol": "local",
    "url": "/path/to/data",
    "username": "optional",
    "password": "optional",
    "bucket": "optional",
    "region": "optional"
  }
}
```

**响应**:
```json
{
  "status": "success",
  "data": {
    "session_id": "uuid-string",
    "protocol": "local",
    "connected": true
  }
}
```

### DELETE /storage/disconnect/{session_id}
断开存储连接

**响应**:
```json
{
  "status": "success",
  "data": "Disconnected"
}
```

### GET /storage/sessions
列出所有活动会话

**响应**:
```json
{
  "status": "success",
  "data": ["session-id-1", "session-id-2"]
}
```

## 文件系统操作

### POST /storage/{session_id}/list
列出目录内容

**请求体**:
```json
{
  "session_id": "uuid-string",
  "path": "optional/path",
  "options": {
    "page_size": 100,
    "marker": "optional",
    "sort_by": "name|size|modified",
    "sort_order": "asc|desc"
  }
}
```

**响应**:
```json
{
  "status": "success",
  "data": {
    "files": [
      {
        "filename": "example.csv",
        "basename": "example.csv",
        "lastmod": "2024-01-01T00:00:00Z",
        "size": "1024",
        "type": "file",
        "mime": "text/csv",
        "etag": "optional"
      }
    ],
    "has_more": false,
    "next_marker": null,
    "total_count": "1",
    "path": "/"
  }
}
```

### POST /storage/{session_id}/file/content
获取文件内容

**请求体**:
```json
{
  "session_id": "uuid-string",
  "path": "file/path",
  "start": 0,
  "length": 1024
}
```

**响应**:
```json
{
  "status": "success",
  "data": {
    "content": [/* byte array */],
    "size": 1024,
    "mime_type": "text/plain",
    "encoding": "utf-8"
  }
}
```

### POST /storage/{session_id}/file/download
下载文件

**请求体**:
```json
{
  "session_id": "uuid-string",
  "file_path": "path/to/file",
  "save_path": "optional/local/path"
}
```

**响应**:
```json
{
  "status": "success",
  "data": {
    "download_id": "uuid-string",
    "file_size": 1048576,
    "started_at": "2024-01-01T00:00:00Z"
  }
}
```

## 压缩包处理

### POST /storage/{session_id}/archive/info
获取压缩包信息

**请求体**:
```json
{
  "file_path": "path/to/archive.zip",
  "max_entries": 1000
}
```

**响应**:
```json
{
  "status": "success",
  "data": {
    "entries": [
      {
        "path": "file.txt",
        "name": "file.txt",
        "size": 1024,
        "compressed_size": 512,
        "modified": "2024-01-01T00:00:00Z",
        "is_directory": false,
        "is_encrypted": false,
        "crc32": 12345678
      }
    ],
    "total_entries": 10,
    "total_uncompressed_size": 10240,
    "total_compressed_size": 5120,
    "format": "Zip",
    "has_more": false
  }
}
```

### POST /storage/{session_id}/archive/file
获取压缩包中的文件

**请求体**:
```json
{
  "archive_path": "path/to/archive.zip",
  "file_path": "internal/file/path",
  "max_size": 1048576,
  "offset": 0
}
```

**响应**:
```json
{
  "status": "success",
  "data": {
    "content": [/* byte array */],
    "is_truncated": false,
    "total_size": 1024,
    "preview_size": 1024
  }
}
```

## WebSocket 消息

WebSocket 连接到 `ws://localhost:8080/ws`

### 订阅会话
```json
{
  "type": "Subscribe",
  "payload": {
    "session_id": "uuid-string"
  }
}
```

### 进度更新（服务器推送）
```json
{
  "id": "message-id",
  "message": {
    "type": "Progress",
    "payload": {
      "session_id": "uuid-string",
      "file_path": "path/to/file",
      "progress": {
        "current": 1024,
        "total": 2048,
        "percentage": 50.0,
        "speed": 1024000,
        "eta": 1
      }
    }
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 下载完成（服务器推送）
```json
{
  "id": "message-id",
  "message": {
    "type": "DownloadComplete",
    "payload": {
      "session_id": "uuid-string",
      "file_path": "path/to/file",
      "success": true,
      "message": "Download completed successfully"
    }
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## 错误响应

所有错误响应遵循统一格式：

```json
{
  "status": "error",
  "error": "ERROR_TYPE",
  "message": "Human readable error message",
  "details": null
}
```

常见错误类型：
- `STORAGE_ERROR`: 存储操作失败
- `NOT_FOUND`: 资源未找到
- `VALIDATION_ERROR`: 请求验证失败
- `AUTHENTICATION_ERROR`: 认证失败
- `INTERNAL_ERROR`: 内部服务器错误