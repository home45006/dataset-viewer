<template>
  <div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
    <!-- 导航栏 -->
    <nav class="bg-white dark:bg-gray-800 shadow-sm">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center space-x-4">
            <h1 class="text-xl font-bold text-gray-900 dark:text-white">Dataset Viewer</h1>
            <span class="text-sm text-gray-500 dark:text-gray-400">Web版</span>
          </div>
          
          <div class="flex items-center space-x-4">
            <button
              @click="appStore.toggleTheme()"
              class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              :title="$t('common.theme')"
            >
              <svg v-if="appStore.isDark" class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path d="M10 2L13.09 8.26L20 9L14 14.74L15.18 21.02L10 17.77L4.82 21.02L6 14.74L0 9L6.91 8.26L10 2Z"/>
              </svg>
              <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </nav>

    <!-- 主内容区域 -->
    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <!-- 欢迎区域 -->
      <div class="text-center mb-16">
        <h2 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">
          欢迎使用 Dataset Viewer
        </h2>
        <p class="text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto">
          统一的数据集浏览器，支持大文件流式传输、多种存储协议，现在可以在浏览器中直接使用
        </p>
      </div>

      <!-- 功能特性 -->
      <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8 mb-16">
        <div class="card p-6 text-center">
          <div class="w-12 h-12 mx-auto mb-4 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
            <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">多格式支持</h3>
          <p class="text-gray-600 dark:text-gray-300">支持 CSV、Parquet、Excel、JSON 等多种数据格式，直接在浏览器中预览</p>
        </div>

        <div class="card p-6 text-center">
          <div class="w-12 h-12 mx-auto mb-4 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center">
            <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/>
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">多存储协议</h3>
          <p class="text-gray-600 dark:text-gray-300">支持本地文件、OSS/S3、WebDAV、SSH/SFTP、SMB、HuggingFace Hub 等</p>
        </div>

        <div class="card p-6 text-center">
          <div class="w-12 h-12 mx-auto mb-4 bg-purple-100 dark:bg-purple-900 rounded-full flex items-center justify-center">
            <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">高性能处理</h3>
          <p class="text-gray-600 dark:text-gray-300">支持 GB 级大文件的流式加载和虚拟滚动，流畅浏览大数据集</p>
        </div>
      </div>

      <!-- 快速开始 -->
      <div class="card p-8 text-center">
        <h3 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">开始使用</h3>
        <p class="text-gray-600 dark:text-gray-300 mb-6">
          选择一个存储源开始浏览您的数据集
        </p>
        
        <div class="flex flex-wrap justify-center gap-4">
          <router-link 
            to="/browse" 
            class="btn btn-primary btn-lg inline-flex items-center space-x-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a1 1 0 00-1-1H6a1 1 0 00-1-1V7a1 1 0 011-1h14a1 1 0 011 1v2"/>
            </svg>
            <span>开始浏览</span>
          </router-link>
          
          <a 
            href="/api/docs" 
            target="_blank"
            class="btn btn-outline btn-lg inline-flex items-center space-x-2"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
            <span>API 文档</span>
          </a>
        </div>
      </div>

      <!-- 服务器状态 -->
      <div class="mt-8 text-center">
        <div v-if="serverStatus" class="inline-flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
          <div class="w-2 h-2 bg-green-500 rounded-full"></div>
          <span>后端服务正常运行 (v{{ serverStatus.version }})</span>
        </div>
        <div v-else-if="serverError" class="inline-flex items-center space-x-2 text-sm text-red-600">
          <div class="w-2 h-2 bg-red-500 rounded-full"></div>
          <span>无法连接到后端服务</span>
        </div>
        <div v-else class="inline-flex items-center space-x-2 text-sm text-gray-400">
          <div class="w-2 h-2 bg-gray-400 rounded-full animate-pulse"></div>
          <span>检查后端服务状态...</span>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'

const { t } = useI18n()
const appStore = useAppStore()

// 服务器状态
const serverStatus = ref<any>(null)
const serverError = ref(false)

// 检查服务器状态
const checkServerStatus = async () => {
  try {
    const response = await fetch('/api/status')
    if (response.ok) {
      const data = await response.json()
      if (data.status === 'success') {
        serverStatus.value = data.data
        serverError.value = false
      } else {
        serverError.value = true
      }
    } else {
      serverError.value = true
    }
  } catch (error) {
    console.error('Failed to check server status:', error)
    serverError.value = true
  }
}

onMounted(() => {
  appStore.initTheme()
  checkServerStatus()
})
</script>