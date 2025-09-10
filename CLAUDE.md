# CLAUDE.md

此文件为 Claude Code (claude.ai/code) 在此代码库中工作时提供指导。

## 项目概览

Dataset Viewer 是一个跨平台的 Tauri 应用程序，用于统一的数据集浏览，具备大文件流式传输能力。这是一个 100% AI 生成的项目，使用 React + TypeScript（前端）和 Tauri/Rust（后端）。

**主要功能：**
- 处理 100GB+ 文件，支持虚拟滚动和分块加载
- 多协议支持：WebDAV、SSH/SFTP、SMB/CIFS、S3、本地文件、HuggingFace Hub
- 直接预览压缩包（ZIP/TAR），无需解压
- 实时搜索，支持正则表达式和高亮显示
- 多格式支持：Parquet、Excel、CSV、JSON、代码文件（带语法高亮）

## 开发命令

### 前端开发
```bash
# 启动开发服务器
pnpm dev

# 构建前端
pnpm build

# 类型检查
pnpm type-check

# 格式化代码（前端 + 后端）
pnpm format

# 检查并格式化（前端 + 后端）
pnpm check
```

### Tauri 命令
```bash
# 启动 Tauri 开发服务器
pnpm tauri:dev

# 构建应用程序
pnpm package

# 构建调试版本
pnpm package:debug

# 跨平台构建
pnpm package:windows
pnpm package:all
```

### 代码质量
- **代码检查/格式化工具**：使用 Biome 而非 ESLint/Prettier
- **前端格式化**：`pnpm format:frontend`
- **后端格式化**：`pnpm format:backend`（Rust cargo fmt）
- **提交前钩子**：使用 Husky 和 lint-staged 自动格式化

## 架构概览

### 前端结构 (`src/`)
- **components/**：按功能组织的 React 组件
  - `ConnectionPanel/`：连接管理界面
  - `FileBrowser/`：文件导航和列表
  - `FileViewer/`：文件内容显示（带虚拟滚动）
  - `common/`：共享 UI 组件
- **services/**：业务逻辑和存储客户端
  - `storage/`：特定协议存储客户端（WebDAV、S3、HuggingFace 等）
  - `connectionStorage.ts`：安全凭证管理
  - `compression.ts`：压缩包处理
- **hooks/**：自定义 React 钩子
- **types/**：TypeScript 类型定义
- **i18n/**：国际化（中文/英文）

### 后端结构 (`src-tauri/`)
- **Rust 二进制目标**：
  - `dataset-viewer`：主应用程序
  - `export-bindings`：TypeScript 绑定生成器
- **核心功能**：HTTP 客户端、文件流、压缩包处理、协议 URL 解析

### 存储架构
- **BaseStorageClient**：统一 `toProtocolUrl()` 方法的抽象基类
- **协议 URL 标准**：
  - OSS：`oss://bucket/path`
  - WebDAV：`webdav://host/path`
  - HuggingFace：`huggingface://owner:dataset/path`
  - 本地：`file:///absolute/path`
- **统一后端**：所有文件操作都一致地解析协议 URL

## 开发指南

### TypeScript/React
- 使用严格的 TypeScript 类型
- 按功能组织组件，使用组合模式
- 对于 >100 项使用虚拟滚动，对于 >1MB 文件使用分块加载
- 将所有 UI 文本包装在 i18n 翻译函数中
- 使用 React 钩子 + localStorage 持久化进行状态管理

### Rust/Tauri
- 使用异步命令和官方 Tauri 插件
- 遵循 Tauri 安全最佳实践
- 所有存储客户端都继承自 BaseStorageClient
- 将路径处理逻辑保留在各自的客户端内

### 样式
- 使用 Tailwind CSS 实用类
- 支持深色/浅色主题
- 响应式设计原则

### 代码风格
- **Biome 配置**：2 空格缩进，100 字符行宽，单引号
- **导入风格**：对 TypeScript 类型使用 import type
- **格式化**：通过提交前钩子自动执行

## 测试和构建流程

1. **提交前**：运行 `pnpm check` 格式化和检查所有代码
2. **类型安全**：运行 `pnpm type-check` 进行 TypeScript 验证
3. **构建验证**：运行 `pnpm build` 确保编译成功
4. **后端绑定**：运行 `cd src-tauri && cargo run --bin export-bindings` 更新 TypeScript 绑定

## 重要说明

- **包管理器**：使用 PNPM（版本 10.14.0+）
- **Tauri 版本**：2.0（最新主要版本）
- **预构建步骤**：自动导出 Rust->TypeScript 绑定
- **跨平台**：支持 Windows、macOS、Linux
- **性能优化**：针对大文件和数据集进行优化
- **AI 生成**：整个代码库通过 AI 辅助创建

## 提交消息格式

遵循 Conventional Commits 规范：
- `feat(scope): description` - 新功能
- `fix(scope): description` - 错误修复
- `docs(scope): description` - 文档更改
- `refactor(scope): description` - 代码重构
- 专注于用户影响，使用英语，保持简洁