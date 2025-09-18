<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
    <!-- 顶部导航 -->
    <nav class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between items-center h-16">
          <div class="flex items-center space-x-4">
            <router-link to="/" class="text-blue-600 hover:text-blue-800">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
              </svg>
            </router-link>
            <h1 class="text-xl font-semibold text-gray-900 dark:text-white">数据集浏览器</h1>
          </div>

          <div class="flex items-center space-x-2">
            <span v-if="isConnected" class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
              <div class="w-1.5 h-1.5 bg-green-500 rounded-full mr-1.5"></div>
              已连接
            </span>
            <button
              @click="appStore.toggleTheme()"
              class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
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

    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="grid lg:grid-cols-4 gap-8">
        <!-- 左侧连接面板 -->
        <div class="lg:col-span-1">
          <div class="card sticky top-8">
            <div class="card-header">
              <div class="flex items-center justify-between">
                <h3 class="text-lg font-medium text-gray-900 dark:text-white">存储连接</h3>
                <div class="flex items-center space-x-2">
                  <!-- 缓存状态指示器 -->
                  <div
                    v-if="cacheInfo.hasCache"
                    class="flex items-center text-xs text-green-600 dark:text-green-400"
                    :title="`已缓存配置: ${ConnectionCacheService.getStorageTypeName(cacheInfo.storageType || '')} (${cacheInfo.lastUsed})`"
                  >
                    <svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                      <path d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z"/>
                    </svg>
                    已缓存
                  </div>
                </div>
              </div>
            </div>

            <div class="card-body space-y-4">
              <!-- 缓存提示和控制 -->
              <div v-if="cacheInfo.hasCache" class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <p class="text-sm text-blue-800 dark:text-blue-200 mb-1">
                      检测到上次保存的配置
                    </p>
                    <p class="text-xs text-blue-600 dark:text-blue-400">
                      {{ ConnectionCacheService.getStorageTypeName(cacheInfo.storageType || '') }} • {{ cacheInfo.lastUsed }}
                    </p>
                    <div class="flex items-center mt-2 space-x-2">
                      <button
                        @click="loadFromCache"
                        class="text-xs bg-blue-600 text-white px-2 py-1 rounded hover:bg-blue-700"
                      >
                        加载配置
                      </button>
                      <button
                        @click="clearCache"
                        class="text-xs text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200"
                      >
                        清除缓存
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 存储类型选择 -->
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  存储类型
                </label>
                <select v-model="selectedStorageType" class="input">
                  <option value="local">本地文件</option>
                  <option value="oss">对象存储 (OSS/S3)</option>
                  <option value="webdav">WebDAV</option>
                  <option value="ssh">SSH/SFTP</option>
                  <option value="smb">SMB/CIFS</option>
                  <option value="huggingface">HuggingFace Hub</option>
                </select>
              </div>

              <!-- 本地文件配置 -->
              <div v-if="selectedStorageType === 'local'">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  根目录路径
                </label>
                <input
                  v-model="connectionConfig.url"
                  type="text"
                  class="input"
                  placeholder="/path/to/data"
                />
              </div>

              <!-- OSS 配置 -->
              <div v-if="selectedStorageType === 'oss'" class="space-y-4">
                <!-- 云服务商和地域选择 -->
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      云服务商
                    </label>
                    <select v-model="ossConfig.platform" @change="handlePlatformChange" class="input">
                      <option value="aliyun">阿里云 OSS</option>
                      <option value="tencent">腾讯云 COS</option>
                      <option value="aws">AWS S3</option>
                      <option value="huawei">华为云 OBS</option>
                      <option value="minio">MinIO</option>
                      <option value="custom">自定义</option>
                    </select>
                  </div>

                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      {{ ossConfig.platform === 'custom' ? '自定义端点' : '地域' }}
                    </label>
                    <select
                      v-if="ossConfig.platform !== 'custom'"
                      v-model="ossConfig.region"
                      @change="handleRegionChange"
                      class="input"
                    >
                      <option v-for="region in getCurrentRegions()" :key="region.id" :value="region.id">
                        {{ region.name }}
                      </option>
                    </select>
                    <input
                      v-else
                      v-model="ossConfig.customEndpoint"
                      @input="handleCustomEndpointChange"
                      type="url"
                      class="input"
                      placeholder="https://your-s3-compatible-endpoint.com"
                    />
                  </div>
                </div>

                <!-- 存储桶 -->
                <div>
                  <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    存储桶/Bucket
                  </label>
                  <input
                    v-model="connectionConfig.bucket"
                    type="text"
                    class="input"
                    placeholder="my-bucket"
                  />
                </div>

                <!-- 访问密钥 -->
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      访问密钥 (Access Key)
                    </label>
                    <input
                      v-model="connectionConfig.access_key"
                      type="text"
                      class="input"
                      placeholder="AccessKey ID"
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                      私有密钥 (Secret Key)
                    </label>
                    <input
                      v-model="connectionConfig.secret_key"
                      type="password"
                      class="input"
                      placeholder="Secret Access Key"
                    />
                  </div>
                </div>

                <!-- 显示当前端点 -->
                <div class="bg-gray-50 dark:bg-gray-800 p-3 rounded-lg">
                  <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    当前端点地址
                  </label>
                  <code class="text-xs text-gray-600 dark:text-gray-400">
                    {{ getCurrentEndpoint() }}
                  </code>
                </div>
              </div>

              <!-- WebDAV 配置 -->
              <div v-if="selectedStorageType === 'webdav'" class="space-y-3">
                <div>
                  <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    WebDAV URL
                  </label>
                  <input
                    v-model="connectionConfig.url"
                    type="text"
                    class="input"
                    placeholder="https://example.com/webdav"
                  />
                </div>
                <div class="grid grid-cols-2 gap-2">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      用户名
                    </label>
                    <input
                      v-model="connectionConfig.username"
                      type="text"
                      class="input"
                    />
                  </div>
                  <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      密码
                    </label>
                    <input
                      v-model="connectionConfig.password"
                      type="password"
                      class="input"
                    />
                  </div>
                </div>
              </div>

              <!-- 连接按钮 -->
              <button
                @click="connect"
                :disabled="isConnecting"
                class="w-full btn"
                :class="isConnected ? 'btn-secondary' : 'btn-primary'"
              >
                <svg v-if="isConnecting" class="w-4 h-4 mr-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                {{ isConnecting ? '连接中...' : isConnected ? '断开连接' : '连接' }}
              </button>
            </div>
          </div>
        </div>

        <!-- 右侧文件浏览区域 -->
        <div class="lg:col-span-3">
          <!-- 未连接状态 -->
          <div v-if="!isConnected" class="card p-12 text-center">
            <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center">
              <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
              </svg>
            </div>
            <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">选择存储源</h3>
            <p class="text-gray-500 dark:text-gray-400">请先在左侧配置并连接到一个存储源开始浏览文件</p>
          </div>

          <!-- 文件列表 -->
          <div v-else class="card">
            <div class="card-header flex items-center justify-between">
              <div class="flex items-center space-x-4">
                <h3 class="text-lg font-medium text-gray-900 dark:text-white">文件浏览</h3>
                <nav class="text-sm text-gray-500">
                  <span>/</span>
                  <span v-for="(segment, index) in pathSegments" :key="index">
                    <span class="mx-1">/</span>
                    <span>{{ segment }}</span>
                  </span>
                </nav>
              </div>

              <button
                @click="refreshFiles"
                class="btn btn-outline btn-sm"
                :disabled="isLoading"
              >
                <svg class="w-4 h-4" :class="{ 'animate-spin': isLoading }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                </svg>
                刷新
              </button>
            </div>

            <div class="card-body p-0">
              <!-- 加载状态 -->
              <div v-if="isLoading" class="p-8 text-center">
                <div class="inline-flex items-center space-x-2">
                  <div class="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                  <span class="text-gray-600 dark:text-gray-400">加载文件列表...</span>
                </div>
              </div>

              <!-- 文件列表 -->
              <div v-else-if="files.length > 0" class="table-container">
                <table>
                  <thead>
                    <tr>
                      <th class="w-8"></th>
                      <th>名称</th>
                      <th class="w-24">大小</th>
                      <th class="w-32">修改时间</th>
                      <th class="w-20">操作</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="file in files"
                      :key="file.filename"
                      @click="handleFileClick(file)"
                      class="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800"
                    >
                      <td>
                        <div class="flex items-center justify-center">
                          <svg v-if="file.type === 'directory'" class="w-5 h-5 text-blue-500" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                          </svg>
                          <svg v-else class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                          </svg>
                        </div>
                      </td>
                      <td class="font-medium text-gray-900 dark:text-white">{{ file.basename }}</td>
                      <td class="text-sm text-gray-500">
                        {{ file.type === 'file' ? formatFileSize(parseInt(file.size)) : '-' }}
                      </td>
                      <td class="text-sm text-gray-500">{{ formatDate(file.lastmod) }}</td>
                      <td>
                        <button
                          v-if="file.type === 'file'"
                          @click.stop="downloadFile(file)"
                          class="text-blue-600 hover:text-blue-800 text-sm"
                        >
                          下载
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>

              <!-- 空文件夹 -->
              <div v-else class="p-8 text-center">
                <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 dark:bg-gray-800 rounded-full flex items-center justify-center">
                  <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a1 1 0 00-1-1H6a1 1 0 00-1-1V7a1 1 0 011-1h14a1 1 0 011 1v2"/>
                  </svg>
                </div>
                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">文件夹为空</h3>
                <p class="text-gray-500 dark:text-gray-400">此目录中没有任何文件或文件夹</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { ConnectionCacheService } from '@/services/storage/ConnectionCacheService'

const appStore = useAppStore()

// 连接状态
const isConnected = ref(false)
const isConnecting = ref(false)
const sessionId = ref<string>('')

// 存储配置
const selectedStorageType = ref('local')
const connectionConfig = reactive({
  url: '/tmp',
  endpoint: '',
  bucket: '',
  access_key: '',
  secret_key: '',
  username: '',
  password: '',
})

// 文件浏览状态
const isLoading = ref(false)
const currentPath = ref('')
const files = ref<any[]>([])

// OSS配置状态
const ossConfig = reactive({
  platform: 'aliyun',
  region: 'cn-hangzhou',
  customEndpoint: ''
})

// 缓存相关状态
const cacheInfo = ref(ConnectionCacheService.getCacheInfo())

// OSS平台配置数据
const ossPlatforms = {
  aliyun: {
    name: '阿里云 OSS',
    defaultRegion: 'cn-hangzhou',
    regions: [
      { id: 'cn-hangzhou', name: '华东1（杭州）', endpoint: 'https://oss-cn-hangzhou.aliyuncs.com' },
      { id: 'cn-shanghai', name: '华东2（上海）', endpoint: 'https://oss-cn-shanghai.aliyuncs.com' },
      { id: 'cn-qingdao', name: '华北1（青岛）', endpoint: 'https://oss-cn-qingdao.aliyuncs.com' },
      { id: 'cn-beijing', name: '华北2（北京）', endpoint: 'https://oss-cn-beijing.aliyuncs.com' },
    ]
  },
  tencent: {
    name: '腾讯云 COS',
    defaultRegion: 'ap-beijing',
    regions: [
      { id: 'ap-beijing', name: '北京', endpoint: 'https://cos.ap-beijing.myqcloud.com' },
      { id: 'ap-shanghai', name: '上海', endpoint: 'https://cos.ap-shanghai.myqcloud.com' },
      { id: 'ap-guangzhou', name: '广州', endpoint: 'https://cos.ap-guangzhou.myqcloud.com' },
    ]
  },
  aws: {
    name: 'AWS S3',
    defaultRegion: 'us-east-1',
    regions: [
      { id: 'us-east-1', name: '默认区域 - US East (N. Virginia)', endpoint: 'https://s3.amazonaws.com' },
      { id: 'us-west-1', name: 'US West (N. California)', endpoint: 'https://s3.us-west-1.amazonaws.com' },
    ]
  },
  huawei: {
    name: '华为云 OBS',
    defaultRegion: 'cn-north-1',
    regions: [
      { id: 'cn-north-1', name: '华北-北京一', endpoint: 'https://obs.cn-north-1.myhuaweicloud.com' },
      { id: 'cn-east-2', name: '华东-上海二', endpoint: 'https://obs.cn-east-2.myhuaweicloud.com' },
    ]
  },
  minio: {
    name: 'MinIO',
    defaultRegion: 'us-east-1',
    regions: [
      { id: 'us-east-1', name: 'Default Region', endpoint: 'http://localhost:9000' }
    ]
  },
  custom: {
    name: '自定义',
    defaultRegion: '',
    regions: []
  }
}

// 计算属性
const pathSegments = computed(() => {
  return currentPath.value.split('/').filter(Boolean)
})

// 组件挂载时检查缓存
onMounted(() => {
  cacheInfo.value = ConnectionCacheService.getCacheInfo()
})

// 从缓存加载配置
const loadFromCache = () => {
  const cached = ConnectionCacheService.loadConnectionConfig()
  if (cached) {
    selectedStorageType.value = cached.storageType

    // 加载连接配置（非敏感信息）
    Object.assign(connectionConfig, {
      url: cached.connectionConfig.url,
      endpoint: cached.connectionConfig.endpoint,
      bucket: cached.connectionConfig.bucket,
      access_key: cached.connectionConfig.access_key,
      username: cached.connectionConfig.username,
      // 敏感信息不从缓存加载
      secret_key: '',
      password: '',
    })

    // 加载OSS配置
    Object.assign(ossConfig, cached.ossConfig)

    // 如果是OSS类型，更新端点
    if (cached.storageType === 'oss') {
      updateConnectionEndpoint()
    }

    console.log('已从缓存加载连接配置')
  }
}

// 清除缓存
const clearCache = () => {
  ConnectionCacheService.clearCache()
  cacheInfo.value = ConnectionCacheService.getCacheInfo()
}

// 保存当前配置到缓存
const saveToCache = () => {
  ConnectionCacheService.saveConnectionConfig(
    selectedStorageType.value,
    connectionConfig,
    ossConfig
  )
  cacheInfo.value = ConnectionCacheService.getCacheInfo()
}

// 重置配置
watch(selectedStorageType, (newType) => {
  Object.assign(connectionConfig, {
    url: newType === 'local' ? '/tmp' : '',
    endpoint: '',
    bucket: '',
    access_key: '',
    secret_key: '',
    username: '',
    password: '',
  })

  // 重置OSS配置
  if (newType === 'oss') {
    ossConfig.platform = 'aliyun'
    ossConfig.region = 'cn-hangzhou'
    ossConfig.customEndpoint = ''
    updateConnectionEndpoint()
  }
})

// 连接存储
const connect = async () => {
  if (isConnected.value) {
    // 断开连接
    try {
      await fetch(`/api/storage/disconnect/${sessionId.value}`, {
        method: 'DELETE',
      })
      isConnected.value = false
      sessionId.value = ''
      files.value = []
    } catch (error) {
      console.error('Disconnect failed:', error)
    }
    return
  }

  isConnecting.value = true
  try {
    const config = {
      protocol: selectedStorageType.value,
      ...connectionConfig,
    }

    const response = await fetch('/api/storage/connect', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ config }),
    })

    const data = await response.json()

    if (data.status === 'success') {
      isConnected.value = true
      sessionId.value = data.data.session_id
      // 连接成功后保存配置到缓存
      saveToCache()
      await loadFiles('')
    } else {
      appStore.setGlobalError(`连接失败: ${data.message}`)
    }
  } catch (error) {
    console.error('Connection failed:', error)
    appStore.setGlobalError('连接失败，请检查网络和后端服务')
  } finally {
    isConnecting.value = false
  }
}

// 加载文件列表
const loadFiles = async (path: string) => {
  if (!isConnected.value) return

  isLoading.value = true
  try {
    const response = await fetch(`/api/storage/${sessionId.value}/list`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId.value,
        path: path || undefined,
        options: {
          page_size: 100,
          sort_by: 'name',
          sort_order: 'asc',
        },
      }),
    })

    const data = await response.json()

    if (data.status === 'success') {
      files.value = data.data.files
      currentPath.value = path
    } else {
      appStore.setGlobalError(`加载文件失败: ${data.message}`)
    }
  } catch (error) {
    console.error('Load files failed:', error)
    appStore.setGlobalError('加载文件失败')
  } finally {
    isLoading.value = false
  }
}

// 刷新文件列表
const refreshFiles = () => {
  loadFiles(currentPath.value)
}

// 处理文件点击
const handleFileClick = (file: any) => {
  if (file.type === 'directory') {
    const newPath = currentPath.value ? `${currentPath.value}/${file.basename}` : file.basename
    loadFiles(newPath)
  } else {
    console.log('File clicked:', file)
  }
}

// 下载文件
const downloadFile = async (file: any) => {
  try {
    const response = await fetch(`/api/storage/${sessionId.value}/file/download`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId.value,
        file_path: file.filename,
      }),
    })

    const data = await response.json()

    if (data.status === 'success') {
      console.log('Download started:', data.data.download_id)
    } else {
      appStore.setGlobalError(`下载失败: ${data.message}`)
    }
  } catch (error) {
    console.error('Download failed:', error)
    appStore.setGlobalError('下载失败')
  }
}

// OSS平台和地域处理方法
const getCurrentRegions = () => {
  const platform = ossPlatforms[ossConfig.platform as keyof typeof ossPlatforms]
  return platform ? platform.regions : []
}

const getCurrentEndpoint = () => {
  if (ossConfig.platform === 'custom') {
    return ossConfig.customEndpoint || '请输入自定义端点'
  }

  const platform = ossPlatforms[ossConfig.platform as keyof typeof ossPlatforms]
  if (!platform) return ''

  const region = platform.regions.find(r => r.id === ossConfig.region)
  return region ? region.endpoint : platform.regions[0]?.endpoint || ''
}

const handlePlatformChange = () => {
  const platform = ossPlatforms[ossConfig.platform as keyof typeof ossPlatforms]
  if (platform) {
    if (ossConfig.platform === 'custom') {
      ossConfig.region = ''
      ossConfig.customEndpoint = ''
      connectionConfig.endpoint = ''
    } else {
      ossConfig.region = platform.defaultRegion || platform.regions[0]?.id || ''
      ossConfig.customEndpoint = ''
      updateConnectionEndpoint()
    }
  }
}

const handleRegionChange = () => {
  updateConnectionEndpoint()
}

const handleCustomEndpointChange = () => {
  if (ossConfig.platform === 'custom') {
    connectionConfig.endpoint = ossConfig.customEndpoint
  }
}

const updateConnectionEndpoint = () => {
  if (ossConfig.platform !== 'custom') {
    const platform = ossPlatforms[ossConfig.platform as keyof typeof ossPlatforms]
    if (platform) {
      const region = platform.regions.find(r => r.id === ossConfig.region)
      connectionConfig.endpoint = region ? region.endpoint : ''
    }
  }
}

// 工具函数
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

const formatDate = (dateString: string): string => {
  try {
    const date = new Date(dateString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateString
  }
}
</script>