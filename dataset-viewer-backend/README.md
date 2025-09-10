# Dataset Viewer Backend

Dataset Viewer 的后端 Web 服务，提供统一的数据集浏览 API，支持多种存储协议。

## 功能特性

- 🔗 **多协议支持**: Local、OSS/S3、WebDAV、SSH/SFTP、SMB/CIFS、HuggingFace Hub
- 📦 **压缩包处理**: ZIP、TAR、7Z 等格式的直接浏览
- 🚀 **流式传输**: 支持大文件的分块加载和断点续传
- 📡 **实时通信**: WebSocket 支持进度回调和状态更新
- 🔒 **安全可靠**: 会话管理和权限控制

## 快速开始

### 使用 Cargo 运行

```bash
# 开发模式
cargo run --bin server

# 生产模式
cargo run --release --bin server
```

### 使用 Docker 运行

```bash
# 构建镜像
docker build -t dataset-viewer-backend .

# 运行容器
docker run -p 8080:8080 dataset-viewer-backend
```

### 环境变量

- `SERVER_HOST`: 服务器监听地址 (默认: 127.0.0.1)
- `SERVER_PORT`: 服务器端口 (默认: 8080)
- `ALLOW_LOCAL_FILES`: 是否允许访问本地文件 (默认: true)
- `RUST_LOG`: 日志级别 (默认: dataset_viewer_backend=info)

## API 文档

启动服务后访问 http://localhost:8080/api/docs 查看完整的 API 文档。

### 基本使用流程

1. **连接存储**: POST `/api/storage/connect`
2. **浏览文件**: POST `/api/storage/{session_id}/list`
3. **读取内容**: POST `/api/storage/{session_id}/file/content`
4. **WebSocket**: 连接 `ws://localhost:8080/ws` 接收实时更新

### 示例请求

```bash
# 连接本地存储
curl -X POST http://localhost:8080/api/storage/connect \
  -H "Content-Type: application/json" \
  -d '{"config":{"protocol":"local","url":"/path/to/data"}}'

# 列出目录
curl -X POST http://localhost:8080/api/storage/{session_id}/list \
  -H "Content-Type: application/json" \
  -d '{"path":"","options":{"page_size":50}}'
```

## 开发

### 项目结构

```
src/
├── main.rs          # 服务器入口
├── lib.rs           # 库入口
├── api/             # HTTP API 层
├── storage/         # 存储客户端
├── archive/         # 压缩包处理
├── websocket/       # WebSocket 处理
├── utils/           # 工具函数
├── config.rs        # 配置管理
├── error.rs         # 错误处理
└── state.rs         # 应用状态
```

### 添加新的存储协议

1. 在 `src/storage/` 下创建新的客户端文件
2. 实现 `StorageClient` trait
3. 在 `src/storage/manager.rs` 中注册新协议

### 构建和测试

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test

# 构建
cargo build --release
```

## 部署

### 系统服务

创建 systemd 服务文件：

```ini
[Unit]
Description=Dataset Viewer Backend
After=network.target

[Service]
Type=simple
User=dataset-viewer
WorkingDirectory=/opt/dataset-viewer
ExecStart=/opt/dataset-viewer/dataset-viewer-backend
Restart=always
RestartSec=5
Environment=RUST_LOG=dataset_viewer_backend=info
Environment=SERVER_HOST=0.0.0.0
Environment=SERVER_PORT=8080

[Install]
WantedBy=multi-user.target
```

### 反向代理

使用 Nginx 配置反向代理：

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location /api/ {
        proxy_pass http://127.0.0.1:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /ws {
        proxy_pass http://127.0.0.1:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location / {
        root /var/www/dataset-viewer;
        try_files $uri $uri/ /index.html;
    }
}
```

## 安全注意事项

- 本地文件访问仅在开发环境启用
- 所有存储凭证都存储在内存中，不会持久化
- WebSocket 连接需要订阅机制防止信息泄露
- 建议在生产环境中使用 HTTPS 和 WSS

## License

MIT License