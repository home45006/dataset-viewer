# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概览

Dataset Viewer 是一个双架构的数据集浏览应用程序，支持大文件流式传输能力。项目从原始的 Tauri 桌面应用程序演进为支持 **Tauri 桌面版** 和 **Vue Web 前后端分离版** 两种架构的统一解决方案。这是一个 100% AI 生成的项目。

**技术栈：**
- **Tauri 版本**：React + TypeScript（前端）+ Tauri/Rust（桌面应用）
- **Web 版本**：Vue 3 + TypeScript（前端）+ Rust/Axum（后端服务）

**主要功能：**
- 处理 100GB+ 文件，支持虚拟滚动和分块加载
- 多协议支持：WebDAV、SSH/SFTP、SMB/CIFS、S3/OSS、本地文件、HuggingFace Hub
- 直接预览压缩包（ZIP/TAR），无需解压
- 实时搜索，支持正则表达式和高亮显示
- 多格式支持：Parquet、Excel、CSV、JSON、代码文件（带语法高亮）

## 项目整体目录结构

```
dataset-viewer/                           # 项目根目录
├── src/                                  # Tauri 前端源码（React + TypeScript）
├── src-tauri/                            # Tauri 后端源码（Rust）
├── dataset-viewer-frontend/              # Vue Web 前端
├── dataset-viewer-backend/               # Rust Web 后端
├── public/                               # Tauri 静态资源
├── .claude/                              # Claude Code 配置
├── package.json                          # Tauri 版 NPM 配置
├── vite.config.ts                        # Tauri 版 Vite 配置
├── CLAUDE.md                             # 项目指导文档
└── README.md                             # 项目说明
```

## 双架构说明

### Tauri 桌面版
- **目录**: 根目录（`/`）
- **前端**: React + TypeScript + Vite
- **后端**: Tauri Rust 命令
- **端口**: 3000（开发模式）
- **用途**: 跨平台桌面应用程序

### Vue Web 版
- **前端目录**: `dataset-viewer-frontend/`
- **后端目录**: `dataset-viewer-backend/`
- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Axum HTTP 服务器
- **端口**: 前端 3002，后端 8080
- **用途**: Web 浏览器访问的前后端分离应用

## 开发命令

### Tauri 桌面版开发
```bash
# 根目录执行

# 启动 Tauri 开发服务器
pnpm tauri:dev

# 构建前端
pnpm build

# 类型检查
pnpm type-check

# 构建应用程序
pnpm package

# 构建调试版本
pnpm package:debug

# 跨平台构建
pnpm package:windows
pnpm package:all
```

### Vue Web 版开发
```bash
# 启动后端服务器（端口 8080）
cd dataset-viewer-backend
cargo run --bin server

# 启动 Vue 前端（端口 3002）
cd dataset-viewer-frontend
npm run dev

# Vue 前端其他命令
npm run build
npm run preview
npm run type-check
```

### 代码质量
- **代码检查/格式化工具**：使用 Biome 而非 ESLint/Prettier
- **前端格式化**：`pnpm format:frontend`（Tauri 版）
- **后端格式化**：`pnpm format:backend`（Rust cargo fmt）
- **提交前钩子**：使用 Husky 和 lint-staged 自动格式化

## 架构概览

### Tauri 桌面版架构

#### 前端结构 (`src/`)
- **components/**：按功能组织的 React 组件
  - `ConnectionPanel/`：连接管理界面
  - `FileBrowser/`：文件导航和列表
  - `FileViewer/`：文件内容显示（带虚拟滚动）
  - `common/`：共享 UI 组件
- **services/**：业务逻辑和存储客户端
  - `storage/`：特定协议存储客户端（WebDAV、S3、HuggingFace 等）
  - `connectionStorage.ts`：安全凭证管理
  - `compression.ts`：压缩包处理
  - `webCompressionService.ts`：Web API 适配器
- **hooks/**：自定义 React 钩子
- **types/**：TypeScript 类型定义
- **i18n/**：国际化（中文/英文）

#### 后端结构 (`src-tauri/`)
- **Rust 二进制目标**：
  - `dataset-viewer`：主应用程序
  - `export-bindings`：TypeScript 绑定生成器
- **核心功能**：HTTP 客户端、文件流、压缩包处理、协议 URL 解析

### Vue Web 版架构

#### 前端结构 (`dataset-viewer-frontend/`)
```
dataset-viewer-frontend/
├── src/
│   ├── App.vue                           # 主应用组件
│   ├── main.ts                          # 应用入口文件
│   ├── assets/
│   │   └── styles/
│   │       └── main.css                 # 主样式文件
│   ├── i18n/                            # 国际化配置
│   │   ├── index.ts                     # i18n 配置入口
│   │   └── locales/
│   │       ├── en.json                  # 英文翻译
│   │       └── zh.json                  # 中文翻译
│   ├── router/
│   │   └── index.ts                     # Vue Router 路由配置
│   ├── stores/
│   │   └── app.ts                       # Pinia 应用状态管理
│   ├── services/
│   │   └── storage/                     # 存储服务（占位符）
│   │       ├── StorageClient.ts
│   │       └── StorageManager.ts
│   └── views/                           # 页面视图
│       ├── HomeView.vue                 # 首页视图
│       ├── BrowseView.vue               # 浏览视图
│       └── AboutView.vue                # 关于页面
├── public/
│   ├── manifest.json                    # PWA 配置
│   └── vite.svg                         # 静态资源
├── package.json                         # NPM 依赖配置
├── vite.config.ts                       # Vite 构建配置
├── tailwind.config.js                   # Tailwind CSS 配置
├── postcss.config.js                    # PostCSS 配置
├── tsconfig.json                        # TypeScript 配置
└── tsconfig.node.json                   # Node.js TypeScript 配置
```

#### 后端结构 (`dataset-viewer-backend/`)
```
dataset-viewer-backend/
├── src/
│   ├── main.rs                          # 主程序入口
│   ├── lib.rs                           # 库文件
│   ├── config.rs                        # 配置管理
│   ├── error.rs                         # 错误类型定义
│   ├── state.rs                         # 应用状态管理
│   ├── api/                             # HTTP API 层
│   │   ├── mod.rs                       # API 模块入口
│   │   ├── routes.rs                    # 路由定义
│   │   ├── types.rs                     # API 类型定义
│   │   └── handlers/                    # API 处理器
│   │       ├── mod.rs                   # 健康检查和状态
│   │       ├── storage.rs               # 存储连接管理
│   │       └── archive.rs               # 压缩包处理
│   ├── storage/                         # 存储客户端层
│   │   ├── mod.rs                       # 存储模块入口
│   │   ├── traits.rs                    # 存储 trait 定义
│   │   ├── types.rs                     # 存储类型定义
│   │   ├── manager.rs                   # 存储管理器
│   │   ├── local.rs                     # 本地文件系统
│   │   ├── webdav.rs                    # WebDAV 协议
│   │   ├── oss.rs                       # 对象存储服务（腾讯云 COS）
│   │   ├── huggingface.rs               # HuggingFace Hub
│   │   ├── smb.rs                       # SMB/CIFS 协议
│   │   └── ssh.rs                       # SSH/SFTP 协议
│   ├── archive/                         # 压缩包处理层
│   │   ├── mod.rs                       # 压缩包模块入口
│   │   ├── types.rs                     # 压缩包类型定义
│   │   └── handlers.rs                  # 压缩包 API 处理器
│   ├── websocket/                       # WebSocket 层
│   │   ├── mod.rs                       # WebSocket 模块入口
│   │   ├── handler.rs                   # WebSocket 处理器
│   │   ├── manager.rs                   # WebSocket 连接管理
│   │   └── messages.rs                  # WebSocket 消息类型
│   ├── utils/                           # 工具模块
│   │   ├── mod.rs                       # 工具模块入口
│   │   ├── http_downloader.rs           # HTTP 下载工具
│   │   └── path_utils.rs                # 路径处理工具
│   └── bin/
│       └── test_zip.rs                  # 测试二进制文件
├── docs/
│   └── api.md                           # API 文档
├── Cargo.toml                           # Rust 依赖配置
├── Cargo.lock                           # Rust 依赖锁文件
├── Dockerfile                           # Docker 配置
└── README.md                            # 项目说明
```

### API 路由结构
```
/health                                    # 健康检查
/status                                    # 服务器状态
/storage/connect                           # 连接存储
/storage/disconnect/:session_id            # 断开连接
/storage/sessions                          # 会话列表
/storage/sessions/:session_id              # 获取会话信息
/storage/:session_id/list                  # 列出目录
/storage/:session_id/file/content          # 获取文件内容
/storage/:session_id/file/info             # 获取文件信息
/storage/:session_id/file/download         # 下载文件
/storage/:session_id/archive/info          # 压缩包信息
/storage/:session_id/archive/file          # 压缩包文件内容
/docs                                      # API 文档
/version                                   # 版本信息
```

### 存储架构
- **统一协议 URL 标准**：
  - OSS：`oss://bucket/path`
  - WebDAV：`webdav://host/path`
  - HuggingFace：`huggingface://owner:dataset/path`
  - 本地：`file:///absolute/path`
- **会话管理**：基于 Session ID 的连接状态管理
- **流式传输**：大文件分块读取和传输

## 开发指南

### 跨平台兼容性
- **环境检测**：使用 `window.__TAURI__` 检测运行环境
- **服务适配**：UnifiedCompressionService 自动选择 Tauri 命令或 Web API
- **API 统一**：两个版本使用相同的存储客户端接口

### TypeScript/React (Tauri 版)
- 使用严格的 TypeScript 类型
- 按功能组织组件，使用组合模式
- 对于 >100 项使用虚拟滚动，对于 >1MB 文件使用分块加载
- 将所有 UI 文本包装在 i18n 翻译函数中
- 使用 React 钩子 + localStorage 持久化进行状态管理

### Vue 3/TypeScript (Web 版)
- 使用 Composition API 和 `<script setup>` 语法
- Pinia 进行状态管理
- Vue Router 进行路由管理
- 响应式数据和计算属性
- TypeScript 严格类型检查

### Rust 后端开发
- **Tauri 命令**：异步命令和官方 Tauri 插件
- **Web 服务器**：Axum 框架，异步处理
- **存储客户端**：统一的 trait 接口
- **错误处理**：统一的错误类型和处理机制
- **会话管理**：线程安全的状态管理

### 样式
- 使用 Tailwind CSS 实用类
- 支持深色/浅色主题
- 响应式设计原则
- 组件级样式隔离

### 代码风格
- **Biome 配置**：2 空格缩进，100 字符行宽，单引号
- **导入风格**：对 TypeScript 类型使用 import type
- **格式化**：通过提交前钩子自动执行
- **Rust 格式化**：使用 cargo fmt

## 测试和构建流程

### Tauri 版本
1. **提交前**：运行 `pnpm check` 格式化和检查所有代码
2. **类型安全**：运行 `pnpm type-check` 进行 TypeScript 验证
3. **构建验证**：运行 `pnpm build` 确保编译成功
4. **后端绑定**：运行 `cd src-tauri && cargo run --bin export-bindings` 更新 TypeScript 绑定

### Vue Web 版本
1. **后端构建**：`cd dataset-viewer-backend && cargo build`
2. **前端构建**：`cd dataset-viewer-frontend && npm run build`
3. **类型检查**：`npm run type-check`
4. **服务启动**：后端先启动，前端再启动

## 配置说明

### 端口配置
- **Tauri 开发模式**：3000
- **Vue 前端**：3002
- **Rust 后端**：8080
- **WebDAV 代理**：3001（如果需要）

### 环境变量
- `TAURI_DEV_HOST`：Tauri 开发主机
- `PORT`：前端端口覆盖

### Vite 配置差异
- **Tauri 版** (`vite.config.ts`)：React 插件，Tauri 优化
- **Vue 版** (`dataset-viewer-frontend/vite.config.ts`)：Vue 插件，API 代理

## 重要说明

- **包管理器**：
  - Tauri 版：PNPM（版本 10.14.0+）
  - Vue 版：NPM
- **Tauri 版本**：2.0（最新主要版本）
- **Vue 版本**：3.x（Composition API）
- **跨平台**：Tauri 支持 Windows、macOS、Linux；Web 版支持所有现代浏览器
- **性能优化**：针对大文件和数据集进行优化
- **AI 生成**：整个代码库通过 AI 辅助创建

## 架构演进说明

项目从单一的 Tauri 桌面应用演进为双架构解决方案：

1. **原始架构**：Pure Tauri 桌面应用
2. **过渡阶段**：添加 Web API 兼容层
3. **当前架构**：完整的双版本支持
   - 保留 Tauri 桌面版的全部功能
   - 新增 Vue Web 版的独立前后端
   - 共享核心业务逻辑和存储客户端

## 提交消息格式

遵循 Conventional Commits 规范：
- `feat(tauri): description` - Tauri 版新功能
- `feat(web): description` - Web 版新功能
- `feat(shared): description` - 共享功能
- `fix(scope): description` - 错误修复
- `docs(scope): description` - 文档更改
- `refactor(scope): description` - 代码重构
- 专注于用户影响，使用英语，保持简洁