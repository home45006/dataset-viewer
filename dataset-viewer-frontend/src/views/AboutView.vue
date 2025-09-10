<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 py-12">
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="text-center mb-12">
        <router-link to="/" class="inline-flex items-center text-blue-600 hover:text-blue-800 mb-4">
          <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
          </svg>
          返回首页
        </router-link>
        
        <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">关于 Dataset Viewer</h1>
        <p class="text-xl text-gray-600 dark:text-gray-300">
          现代化的统一数据集浏览器 Web 版本
        </p>
      </div>

      <div class="grid md:grid-cols-2 gap-8">
        <!-- 项目信息 -->
        <div class="card">
          <div class="card-header">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white">项目信息</h2>
          </div>
          <div class="card-body space-y-4">
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">版本</dt>
              <dd class="mt-1 text-lg text-gray-900 dark:text-white">v1.4.0</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">架构</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">Vue 3 + Rust (Axum) Web 应用</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">前身</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">基于 Tauri + React 的桌面应用</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">开发方式</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">100% AI 辅助开发</dd>
            </div>
          </div>
        </div>

        <!-- 技术栈 -->
        <div class="card">
          <div class="card-header">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white">技术栈</h2>
          </div>
          <div class="card-body space-y-4">
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">前端</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">
                Vue 3, TypeScript, Tailwind CSS, Vite
              </dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">后端</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">
                Rust, Axum, Tokio, SQLite
              </dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">部署</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">
                Docker, Nginx, 云服务器
              </dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">协议支持</dt>
              <dd class="mt-1 text-gray-900 dark:text-white">
                HTTP, WebSocket, REST API
              </dd>
            </div>
          </div>
        </div>
      </div>

      <!-- 功能特性 -->
      <div class="card mt-8">
        <div class="card-header">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">功能特性</h2>
        </div>
        <div class="card-body">
          <div class="grid md:grid-cols-2 gap-6">
            <div class="space-y-4">
              <h3 class="text-lg font-medium text-gray-900 dark:text-white">存储协议支持</h3>
              <ul class="space-y-2 text-gray-600 dark:text-gray-300">
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>本地文件系统</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>OSS/S3 对象存储</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>WebDAV 服务器</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>SSH/SFTP</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>SMB/CIFS 网络共享</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>HuggingFace Hub</span>
                </li>
              </ul>
            </div>

            <div class="space-y-4">
              <h3 class="text-lg font-medium text-gray-900 dark:text-white">数据格式支持</h3>
              <ul class="space-y-2 text-gray-600 dark:text-gray-300">
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>Parquet 列存储</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>Excel (XLSX/XLS)</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>CSV/TSV 文本</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>JSON/JSONL</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>代码文件 (带高亮)</span>
                </li>
                <li class="flex items-center space-x-2">
                  <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                  </svg>
                  <span>压缩包 (ZIP/TAR)</span>
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      <!-- 架构优势 -->
      <div class="card mt-8">
        <div class="card-header">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">Web 版优势</h2>
        </div>
        <div class="card-body">
          <div class="grid md:grid-cols-3 gap-6">
            <div class="text-center">
              <div class="w-12 h-12 mx-auto mb-4 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9v-9m0-9v9"/>
                </svg>
              </div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">跨平台访问</h3>
              <p class="text-gray-600 dark:text-gray-300">
                支持所有现代浏览器，无需安装客户端，随时随地访问数据
              </p>
            </div>

            <div class="text-center">
              <div class="w-12 h-12 mx-auto mb-4 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                </svg>
              </div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">安全可靠</h3>
              <p class="text-gray-600 dark:text-gray-300">
                会话管理和权限控制，保证数据安全，支持 HTTPS 加密传输
              </p>
            </div>

            <div class="text-center">
              <div class="w-12 h-12 mx-auto mb-4 bg-purple-100 dark:bg-purple-900 rounded-full flex items-center justify-center">
                <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
                </svg>
              </div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">高性能</h3>
              <p class="text-gray-600 dark:text-gray-300">
                Rust 后端保证高性能，WebSocket 实时通信，支持大文件流式处理
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- 链接 -->
      <div class="mt-12 text-center space-y-4">
        <div class="flex justify-center space-x-6">
          <a href="/api/docs" target="_blank" class="text-blue-600 hover:text-blue-800 inline-flex items-center">
            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
            API 文档
          </a>
          <a href="https://github.com/example/dataset-viewer" target="_blank" class="text-blue-600 hover:text-blue-800 inline-flex items-center">
            <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
            </svg>
            源码仓库
          </a>
        </div>
        
        <p class="text-sm text-gray-500 dark:text-gray-400">
          © 2024 Dataset Viewer Web. 基于 MIT 开源协议.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 无需额外逻辑，纯展示页面
</script>