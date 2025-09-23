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
              <h3 class="text-lg font-medium text-gray-900 dark:text-white">存储连接</h3>
            </div>
            
            <div class="card-body space-y-4">
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

              <!-- 缓存信息 -->
              <div v-if="hasDisplayedCache" class="relative mb-4">
                <!-- 头部 -->
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center space-x-2">
                    <div class="w-2 h-2 bg-emerald-500 rounded-full animate-pulse"></div>
                    <span class="text-sm font-medium text-emerald-700 dark:text-emerald-300">
                      {{ showAllCaches ? '所有已保存的配置' : '当前类型的配置' }}
                    </span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">({{ displayedCacheInfo.length }})</span>
                  </div>
                  <div class="flex items-center space-x-2">
                    <!-- 切换显示模式 -->
                    <button
                      v-if="hasAnyCache && allCacheInfo.length > currentStorageTypeCacheInfo.length"
                      @click="showAllCaches = !showAllCaches"
                      class="p-1.5 text-gray-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition-all duration-200"
                      :title="showAllCaches ? '只显示当前类型' : '显示所有类型'"
                    >
                      <svg v-if="showAllCaches" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"/>
                      </svg>
                      <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                      </svg>
                    </button>
                    <!-- 清除按钮 -->
                    <button
                      @click="showAllCaches ? clearCache() : clearSpecificCache(selectedStorageType)"
                      class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all duration-200 group"
                      :title="showAllCaches ? '清除所有缓存' : `清除${ConnectionCacheService.getStorageTypeName(selectedStorageType)}缓存`"
                    >
                      <svg class="w-4 h-4 group-hover:scale-110 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                      </svg>
                    </button>
                  </div>
                </div>

                <!-- 缓存卡片列表 -->
                <div class="space-y-3">
                  <div v-for="cache in displayedCacheInfo" :key="cache.storageType" class="relative group">
                    <!-- 背景装饰 -->
                    <div class="absolute inset-0 bg-gradient-to-r from-emerald-50 to-blue-50 dark:from-emerald-900/10 dark:to-blue-900/10 rounded-xl"></div>
                    <div class="absolute inset-0 bg-white/50 dark:bg-gray-800/50 backdrop-blur-sm rounded-xl border border-emerald-200/50 dark:border-emerald-700/30 group-hover:border-emerald-300 dark:group-hover:border-emerald-600 transition-all duration-200"></div>

                    <!-- 删除按钮 -->
                    <button
                      @click="clearSpecificCache(cache.storageType)"
                      class="absolute top-2 right-2 z-10 p-1 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all duration-200 opacity-0 group-hover:opacity-100"
                      :title="`清除${ConnectionCacheService.getStorageTypeName(cache.storageType)}缓存`"
                    >
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                      </svg>
                    </button>

                    <!-- 缓存卡片内容 -->
                    <button
                      @click="loadSpecificCacheConfig(cache.storageType)"
                      class="w-full relative"
                    >
                      <!-- 左侧装饰条 -->
                      <div class="absolute left-0 top-2 bottom-2 w-1 bg-gradient-to-b from-emerald-400 to-blue-500 rounded-r-full group-hover:w-1.5 transition-all duration-200"></div>

                      <!-- 内容 -->
                      <div class="relative flex items-center space-x-4 p-3 pl-5">
                        <!-- 图标 -->
                        <div class="flex-shrink-0 w-9 h-9 bg-gradient-to-br from-emerald-100 to-blue-100 dark:from-emerald-900/30 dark:to-blue-900/30 rounded-lg flex items-center justify-center group-hover:scale-110 transition-transform duration-200">
                          <svg class="w-4 h-4 text-emerald-600 dark:text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
                          </svg>
                        </div>

                        <!-- 文本信息 -->
                        <div class="flex-1 text-left">
                          <div class="flex items-center space-x-2 mb-1">
                            <h4 class="text-sm font-semibold text-gray-900 dark:text-white group-hover:text-emerald-600 dark:group-hover:text-emerald-400 transition-colors">
                              {{ ConnectionCacheService.getStorageTypeName(cache.storageType) }}
                            </h4>
                            <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-300">
                              快速连接
                            </span>
                          </div>
                          <p class="text-xs text-gray-500 dark:text-gray-400">
                            上次使用：{{ cache.lastUsed }}
                          </p>
                        </div>

                        <!-- 右侧箭头 -->
                        <div class="flex-shrink-0">
                          <svg class="w-4 h-4 text-gray-400 group-hover:text-emerald-500 group-hover:translate-x-1 transition-all duration-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6"/>
                          </svg>
                        </div>
                      </div>
                    </button>
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
                <nav class="flex items-center text-sm text-gray-500 dark:text-gray-400">
                  <!-- 根目录 -->
                  <button
                    @click="navigateToPath('')"
                    class="hover:text-blue-600 dark:hover:text-blue-400 hover:underline"
                    title="返回根目录"
                  >
                    根目录
                  </button>
                  <!-- 路径段 -->
                  <template v-for="(segment, index) in pathSegments" :key="index">
                    <span class="mx-1 text-gray-300 dark:text-gray-600">/</span>
                    <button
                      @click="navigateToPath(pathSegments.slice(0, index + 1).join('/'))"
                      class="hover:text-blue-600 dark:hover:text-blue-400 hover:underline"
                      :title="`进入 ${segment} 目录`"
                    >
                      {{ segment }}
                    </button>
                  </template>
                </nav>
                <!-- 返回上级目录按钮 -->
                <button
                  v-if="currentPath"
                  @click="navigateToParent"
                  class="flex items-center space-x-1 text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 px-2 py-1 rounded hover:bg-blue-50 dark:hover:bg-blue-900/20"
                  title="返回上级目录"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
                  </svg>
                  <span>上级目录</span>
                </button>
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
                    <!-- 返回上级目录选项 -->
                    <tr
                      v-if="currentPath"
                      @click="navigateToParent"
                      class="cursor-pointer hover:bg-blue-50 dark:hover:bg-blue-900/20 border-b border-blue-200 dark:border-blue-800"
                    >
                      <td>
                        <div class="flex items-center justify-center">
                          <svg class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
                          </svg>
                        </div>
                      </td>
                      <td class="font-medium text-blue-600 dark:text-blue-400">.. (返回上级目录)</td>
                      <td class="text-sm text-gray-500">-</td>
                      <td class="text-sm text-gray-500">-</td>
                      <td></td>
                    </tr>
                    <!-- 文件和目录列表 -->
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

    <!-- ZIP文件浏览对话框 -->
    <div
      v-if="isArchiveBrowseOpen"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
      @click="closeArchiveBrowse"
    >
      <div
        class="bg-white dark:bg-gray-800 rounded-lg max-w-6xl w-full max-h-[85vh] flex flex-col"
        @click.stop
      >
        <!-- ZIP浏览头部 -->
        <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center space-x-4">
            <div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                📦 {{ archiveFile?.basename }}
              </h3>
              <p class="text-sm text-gray-500 dark:text-gray-400">
                压缩包内容浏览
              </p>
            </div>
            <!-- 路径导航 -->
            <div v-if="archiveCurrentPath" class="flex items-center text-sm text-gray-600 dark:text-gray-300">
              <button
                @click="archiveCurrentPath = ''"
                class="hover:text-blue-600 dark:hover:text-blue-400"
              >
                根目录
              </button>
              <template v-for="(segment, index) in archiveCurrentPath.split('/').filter(Boolean)" :key="index">
                <span class="mx-1">/</span>
                <button
                  @click="archiveCurrentPath = archiveCurrentPath.split('/').slice(0, index + 1).join('/')"
                  class="hover:text-blue-600 dark:hover:text-blue-400"
                >
                  {{ segment }}
                </button>
              </template>
            </div>
          </div>
          <button
            @click="closeArchiveBrowse"
            class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <!-- ZIP浏览内容 -->
        <div class="flex-1 overflow-auto p-4">
          <div v-if="isLoadingArchive" class="flex items-center justify-center h-64">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          </div>

          <div v-else-if="archiveInfo && archiveInfo.entries" class="space-y-2">
            <!-- 返回上级按钮 -->
            <div v-if="archiveCurrentPath" class="border-b pb-2 mb-4">
              <button
                @click="navigateToArchiveParent"
                class="flex items-center space-x-2 text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
                </svg>
                <span>返回上级目录</span>
              </button>
            </div>

            <!-- 文件列表 -->
            <div class="bg-white dark:bg-gray-900 rounded-lg overflow-hidden shadow-sm border border-gray-200 dark:border-gray-700">
              <table class="w-full">
                <thead class="bg-gray-50 dark:bg-gray-800">
                  <tr>
                    <th class="w-8 p-2"></th>
                    <th class="text-left p-2 font-medium text-gray-700 dark:text-gray-300">名称</th>
                    <th class="text-left p-2 font-medium text-gray-700 dark:text-gray-300">大小</th>
                    <th class="text-left p-2 font-medium text-gray-700 dark:text-gray-300">类型</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="file in getArchiveFilesInPath()"
                    :key="file.path"
                    @click="handleArchiveFileClick(file)"
                    class="cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-800 border-b border-gray-100 dark:border-gray-700"
                  >
                    <td class="p-2">
                      <div class="flex items-center justify-center">
                        <svg v-if="file.isDirectory" class="w-5 h-5 text-blue-500" fill="currentColor" viewBox="0 0 20 20">
                          <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                        </svg>
                        <svg v-else class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                      </div>
                    </td>
                    <td class="p-2 font-medium text-gray-900 dark:text-white">{{ file.name }}</td>
                    <td class="p-2 text-sm text-gray-500">
                      {{ file.isDirectory ? '-' : formatFileSize(file.size) }}
                    </td>
                    <td class="p-2 text-sm text-gray-500">
                      {{ file.isDirectory ? '文件夹' : '文件' }}
                    </td>
                  </tr>
                </tbody>
              </table>

              <div v-if="getArchiveFilesInPath().length === 0" class="p-8 text-center text-gray-500 dark:text-gray-400">
                此目录为空
              </div>
            </div>

            <!-- 压缩包信息 -->
            <div class="mt-4 text-xs text-gray-500 dark:text-gray-400">
              压缩包格式: {{ archiveInfo.format.toUpperCase() }} |
              总文件数: {{ archiveInfo.entries.length }} |
              压缩大小: {{ formatFileSize(archiveInfo.compressed_size || 0) }} |
              原始大小: {{ formatFileSize(archiveInfo.uncompressed_size || 0) }}
            </div>
          </div>

          <div v-else class="text-center text-gray-500 dark:text-gray-400 py-8">
            <div class="flex flex-col items-center space-y-4">
              <svg class="w-16 h-16 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
              </svg>
              <div>
                <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">无法读取压缩包</h3>
                <p class="text-gray-500 dark:text-gray-400">
                  此ZIP文件可能太大或格式不支持
                </p>
                <p class="text-sm text-gray-400 dark:text-gray-500 mt-2">
                  当前版本支持小于64KB的ZIP文件预览
                </p>
              </div>
            </div>
          </div>
        </div>

        <!-- ZIP浏览底部 -->
        <div class="flex items-center justify-end p-4 border-t border-gray-200 dark:border-gray-700 space-x-2">
          <button
            @click="closeArchiveBrowse"
            class="btn btn-secondary btn-sm"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <!-- 文件预览对话框 -->
    <div
      v-if="isPreviewOpen"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
      @click="closeFilePreview"
    >
      <div
        class="bg-white dark:bg-gray-800 rounded-lg max-w-4xl w-full h-[80vh] flex flex-col"
        @click.stop
      >
        <!-- 预览头部 -->
        <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {{ previewFile?.filename }}
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              大小: {{ formatFileSize(parseInt(previewFile?.size || '0')) }}
            </p>
          </div>
          <button
            @click="closeFilePreview"
            class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>

        <!-- 预览内容 -->
        <div class="flex-1 min-h-0 overflow-hidden">
          <div v-if="isLoadingContent" class="flex items-center justify-center h-64">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
          </div>
          <div v-else-if="fileContent" class="h-full">
            <!-- 代码文件使用语法高亮预览 -->
            <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 h-full flex flex-col">
              <div class="mb-2 text-xs text-gray-500 dark:text-gray-400 flex justify-between flex-shrink-0">
                <span>{{ previewFile?.filename }} ({{ fileContent.length }} 字符)</span>
                <span v-if="isCodeFile(previewFile?.filename)">语法高亮: {{ getFileLanguage(previewFile?.filename) }}</span>
              </div>
              <div class="flex-1 overflow-auto min-h-0 border border-gray-200 dark:border-gray-600 rounded custom-scrollbar">
                <pre class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap font-mono leading-relaxed p-4 m-0" v-html="getHighlightedContent()"></pre>
              </div>
            </div>
          </div>
          <div v-else class="flex items-center justify-center h-64">
            <div class="text-center text-gray-500 dark:text-gray-400">
              无法预览此文件内容
            </div>
          </div>
        </div>

        <!-- 预览底部 -->
        <div class="flex items-center justify-end p-4 border-t border-gray-200 dark:border-gray-700 space-x-2">
          <button
            @click.stop="downloadFile(previewFile)"
            class="btn btn-outline btn-sm"
          >
            下载
          </button>
          <button
            @click="closeFilePreview"
            class="btn btn-secondary btn-sm"
          >
            关闭
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { ConnectionCacheService } from '@/services/storage/ConnectionCacheService'
import VirtualizedTextViewer from '@/components/VirtualizedTextViewer.vue'

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

// 文件预览状态
const isPreviewOpen = ref(false)
const previewFile = ref<any>(null)

// ZIP文件浏览状态
const isArchiveBrowseOpen = ref(false)
const archiveFile = ref<any>(null)
const archiveInfo = ref<any>(null)
const isLoadingArchive = ref(false)
const archiveCurrentPath = ref('')
const fileContent = ref('')
const isLoadingContent = ref(false)
const fileLoadOffset = ref(0)
const fileLoadedSize = ref(0)
const fileTotalSize = ref(0)

// 虚拟化查看器配置
const VIRTUAL_VIEWER_THRESHOLD = 100 * 1024 // 100KB
const CHUNK_SIZE = 1024 * 1024 // 1MB per chunk

// OSS配置状态
const ossConfig = reactive({
  platform: 'aliyun',
  region: 'cn-hangzhou',
  customEndpoint: ''
})

// 缓存状态
const allCacheInfo = ref<Array<{ storageType: string; lastUsed: string; config: any }>>([])
const hasAnyCache = computed(() => allCacheInfo.value.length > 0)

// 缓存显示控制
const showAllCaches = ref(false)

// 当前存储类型的缓存信息
const currentStorageTypeCacheInfo = computed(() => {
  return allCacheInfo.value.filter(cache => cache.storageType === selectedStorageType.value)
})
const hasCurrentStorageTypeCache = computed(() => currentStorageTypeCacheInfo.value.length > 0)

// 要显示的缓存信息（根据开关决定）
const displayedCacheInfo = computed(() => {
  return showAllCaches.value ? allCacheInfo.value : currentStorageTypeCacheInfo.value
})
const hasDisplayedCache = computed(() => displayedCacheInfo.value.length > 0)

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
      { id: 'cn-zhangjiakou', name: '华北3（张家口）', endpoint: 'https://oss-cn-zhangjiakou.aliyuncs.com' },
      { id: 'cn-huhehaote', name: '华北5（呼和浩特）', endpoint: 'https://oss-cn-huhehaote.aliyuncs.com' },
      { id: 'cn-wulanchabu', name: '华北6（乌兰察布）', endpoint: 'https://oss-cn-wulanchabu.aliyuncs.com' },
      { id: 'cn-shenzhen', name: '华南1（深圳）', endpoint: 'https://oss-cn-shenzhen.aliyuncs.com' },
      { id: 'cn-heyuan', name: '华南2（河源）', endpoint: 'https://oss-cn-heyuan.aliyuncs.com' },
      { id: 'cn-guangzhou', name: '华南3（广州）', endpoint: 'https://oss-cn-guangzhou.aliyuncs.com' },
      { id: 'cn-chengdu', name: '西南1（成都）', endpoint: 'https://oss-cn-chengdu.aliyuncs.com' },
      { id: 'cn-hongkong', name: '中国香港', endpoint: 'https://oss-cn-hongkong.aliyuncs.com' },
      { id: 'us-west-1', name: '美国西部1（硅谷）', endpoint: 'https://oss-us-west-1.aliyuncs.com' },
      { id: 'us-east-1', name: '美国东部1（弗吉尼亚）', endpoint: 'https://oss-us-east-1.aliyuncs.com' },
      { id: 'ap-southeast-1', name: '亚太东南1（新加坡）', endpoint: 'https://oss-ap-southeast-1.aliyuncs.com' },
      { id: 'ap-southeast-2', name: '亚太东南2（悉尼）', endpoint: 'https://oss-ap-southeast-2.aliyuncs.com' },
      { id: 'ap-southeast-3', name: '亚太东南3（吉隆坡）', endpoint: 'https://oss-ap-southeast-3.aliyuncs.com' },
      { id: 'ap-southeast-5', name: '亚太东南5（雅加达）', endpoint: 'https://oss-ap-southeast-5.aliyuncs.com' },
      { id: 'ap-northeast-1', name: '亚太东北1（日本）', endpoint: 'https://oss-ap-northeast-1.aliyuncs.com' },
      { id: 'ap-south-1', name: '亚太南部1（孟买）', endpoint: 'https://oss-ap-south-1.aliyuncs.com' },
      { id: 'eu-central-1', name: '欧洲中部1（法兰克福）', endpoint: 'https://oss-eu-central-1.aliyuncs.com' },
      { id: 'eu-west-1', name: '英国（伦敦）', endpoint: 'https://oss-eu-west-1.aliyuncs.com' },
      { id: 'me-east-1', name: '中东东部1（迪拜）', endpoint: 'https://oss-me-east-1.aliyuncs.com' }
    ]
  },
  tencent: {
    name: '腾讯云 COS',
    defaultRegion: 'ap-beijing',
    regions: [
      { id: 'ap-beijing-1', name: '北京一区', endpoint: 'https://cos.ap-beijing-1.myqcloud.com' },
      { id: 'ap-beijing', name: '北京', endpoint: 'https://cos.ap-beijing.myqcloud.com' },
      { id: 'ap-nanjing', name: '南京', endpoint: 'https://cos.ap-nanjing.myqcloud.com' },
      { id: 'ap-shanghai', name: '上海', endpoint: 'https://cos.ap-shanghai.myqcloud.com' },
      { id: 'ap-guangzhou', name: '广州', endpoint: 'https://cos.ap-guangzhou.myqcloud.com' },
      { id: 'ap-chengdu', name: '成都', endpoint: 'https://cos.ap-chengdu.myqcloud.com' },
      { id: 'ap-chongqing', name: '重庆', endpoint: 'https://cos.ap-chongqing.myqcloud.com' },
      { id: 'ap-shenzhen-fsi', name: '深圳金融', endpoint: 'https://cos.ap-shenzhen-fsi.myqcloud.com' },
      { id: 'ap-shanghai-fsi', name: '上海金融', endpoint: 'https://cos.ap-shanghai-fsi.myqcloud.com' },
      { id: 'ap-beijing-fsi', name: '北京金融', endpoint: 'https://cos.ap-beijing-fsi.myqcloud.com' },
      { id: 'ap-hongkong', name: '中国香港', endpoint: 'https://cos.ap-hongkong.myqcloud.com' },
      { id: 'ap-singapore', name: '新加坡', endpoint: 'https://cos.ap-singapore.myqcloud.com' },
      { id: 'ap-mumbai', name: '孟买', endpoint: 'https://cos.ap-mumbai.myqcloud.com' },
      { id: 'ap-jakarta', name: '雅加达', endpoint: 'https://cos.ap-jakarta.myqcloud.com' },
      { id: 'ap-seoul', name: '首尔', endpoint: 'https://cos.ap-seoul.myqcloud.com' },
      { id: 'ap-bangkok', name: '曼谷', endpoint: 'https://cos.ap-bangkok.myqcloud.com' },
      { id: 'ap-tokyo', name: '东京', endpoint: 'https://cos.ap-tokyo.myqcloud.com' },
      { id: 'na-siliconvalley', name: '硅谷', endpoint: 'https://cos.na-siliconvalley.myqcloud.com' },
      { id: 'na-ashburn', name: '弗吉尼亚', endpoint: 'https://cos.na-ashburn.myqcloud.com' },
      { id: 'na-toronto', name: '多伦多', endpoint: 'https://cos.na-toronto.myqcloud.com' },
      { id: 'eu-frankfurt', name: '法兰克福', endpoint: 'https://cos.eu-frankfurt.myqcloud.com' },
      { id: 'eu-moscow', name: '莫斯科', endpoint: 'https://cos.eu-moscow.myqcloud.com' }
    ]
  },
  aws: {
    name: 'AWS S3',
    defaultRegion: 'us-east-1',
    regions: [
      { id: 'us-east-1', name: '默认区域 - US East (N. Virginia)', endpoint: 'https://s3.amazonaws.com' },
      { id: 'us-east-2', name: 'US East (Ohio)', endpoint: 'https://s3.us-east-2.amazonaws.com' },
      { id: 'us-west-1', name: 'US West (N. California)', endpoint: 'https://s3.us-west-1.amazonaws.com' },
      { id: 'us-west-2', name: 'US West (Oregon)', endpoint: 'https://s3.us-west-2.amazonaws.com' },
      { id: 'ap-south-1', name: 'Asia Pacific (Mumbai)', endpoint: 'https://s3.ap-south-1.amazonaws.com' },
      { id: 'ap-northeast-1', name: 'Asia Pacific (Tokyo)', endpoint: 'https://s3.ap-northeast-1.amazonaws.com' },
      { id: 'ap-northeast-2', name: 'Asia Pacific (Seoul)', endpoint: 'https://s3.ap-northeast-2.amazonaws.com' },
      { id: 'ap-northeast-3', name: 'Asia Pacific (Osaka)', endpoint: 'https://s3.ap-northeast-3.amazonaws.com' },
      { id: 'ap-southeast-1', name: 'Asia Pacific (Singapore)', endpoint: 'https://s3.ap-southeast-1.amazonaws.com' },
      { id: 'ap-southeast-2', name: 'Asia Pacific (Sydney)', endpoint: 'https://s3.ap-southeast-2.amazonaws.com' },
      { id: 'ca-central-1', name: 'Canada (Central)', endpoint: 'https://s3.ca-central-1.amazonaws.com' },
      { id: 'eu-central-1', name: 'Europe (Frankfurt)', endpoint: 'https://s3.eu-central-1.amazonaws.com' },
      { id: 'eu-west-1', name: 'Europe (Ireland)', endpoint: 'https://s3.eu-west-1.amazonaws.com' },
      { id: 'eu-west-2', name: 'Europe (London)', endpoint: 'https://s3.eu-west-2.amazonaws.com' },
      { id: 'eu-west-3', name: 'Europe (Paris)', endpoint: 'https://s3.eu-west-3.amazonaws.com' },
      { id: 'eu-north-1', name: 'Europe (Stockholm)', endpoint: 'https://s3.eu-north-1.amazonaws.com' },
      { id: 'sa-east-1', name: 'South America (São Paulo)', endpoint: 'https://s3.sa-east-1.amazonaws.com' }
    ]
  },
  huawei: {
    name: '华为云 OBS',
    defaultRegion: 'cn-north-1',
    regions: [
      { id: 'cn-north-1', name: '华北-北京一', endpoint: 'https://obs.cn-north-1.myhuaweicloud.com' },
      { id: 'cn-north-4', name: '华北-北京四', endpoint: 'https://obs.cn-north-4.myhuaweicloud.com' },
      { id: 'cn-north-9', name: '华北-乌兰察布一', endpoint: 'https://obs.cn-north-9.myhuaweicloud.com' },
      { id: 'cn-east-2', name: '华东-上海二', endpoint: 'https://obs.cn-east-2.myhuaweicloud.com' },
      { id: 'cn-east-3', name: '华东-上海一', endpoint: 'https://obs.cn-east-3.myhuaweicloud.com' },
      { id: 'cn-south-1', name: '华南-广州', endpoint: 'https://obs.cn-south-1.myhuaweicloud.com' },
      { id: 'cn-southwest-2', name: '西南-贵阳一', endpoint: 'https://obs.cn-southwest-2.myhuaweicloud.com' },
      { id: 'ap-southeast-1', name: '亚太-香港', endpoint: 'https://obs.ap-southeast-1.myhuaweicloud.com' },
      { id: 'ap-southeast-2', name: '亚太-曼谷', endpoint: 'https://obs.ap-southeast-2.myhuaweicloud.com' },
      { id: 'ap-southeast-3', name: '亚太-新加坡', endpoint: 'https://obs.ap-southeast-3.myhuaweicloud.com' },
      { id: 'af-south-1', name: '非洲-约翰内斯堡', endpoint: 'https://obs.af-south-1.myhuaweicloud.com' }
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

// 判断是否使用虚拟化查看器
const shouldUseVirtualizedViewer = computed(() => {
  if (!previewFile.value) return false

  const fileName = previewFile.value.filename || ''
  const fileSize = parseInt(previewFile.value.size || '0')

  // 检查文件扩展名是否为文本类型
  const textExtensions = ['.txt', '.log', '.csv', '.json', '.xml', '.md', '.yml', '.yaml',
    '.js', '.ts', '.jsx', '.tsx', '.vue', '.html', '.css', '.scss', '.sass', '.py', '.java',
    '.cpp', '.c', '.h', '.go', '.rs', '.php', '.rb', '.sh', '.sql', '.conf', '.ini']

  const isTextFile = textExtensions.some(ext => fileName.toLowerCase().endsWith(ext))

  // 检查是否为可语法高亮的文件类型
  const codeExtensions = ['.js', '.ts', '.jsx', '.tsx', '.vue', '.html', '.css', '.scss', '.sass',
    '.py', '.java', '.cpp', '.c', '.h', '.go', '.rs', '.php', '.rb', '.sh', '.sql', '.json',
    '.xml', '.yml', '.yaml', '.md']

  const isCodeFile = codeExtensions.some(ext => fileName.toLowerCase().endsWith(ext))

  // 对于代码文件，总是使用虚拟化查看器以启用语法高亮
  // 对于其他文本文件，只有大于阈值时才使用虚拟化查看器
  return isTextFile && (isCodeFile || fileSize > VIRTUAL_VIEWER_THRESHOLD || fileContent.value.length > 1000)
})

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

  // 切换存储类型时，自动回到当前类型显示模式
  showAllCaches.value = false
})

// 缓存相关功能
const loadCachedConfig = () => {
  const cached = ConnectionCacheService.loadConnectionConfig(undefined, sessionId.value || undefined)
  if (cached) {
    // 更新存储类型
    selectedStorageType.value = cached.storageType

    // 重置并更新连接配置
    Object.assign(connectionConfig, {
      url: cached.storageType === 'local' ? '/tmp' : '',
      endpoint: '',
      bucket: '',
      access_key: '',
      secret_key: '',
      username: '',
      password: '',
    }, cached.connectionConfig)

    // 更新OSS配置
    Object.assign(ossConfig, cached.ossConfig)

    // 更新连接端点
    updateConnectionEndpoint()

    // 显示成功提示
    appStore.setGlobalError('')  // 清除可能存在的错误信息
    console.log('已自动填写缓存的连接配置')
  } else {
    console.log('没有找到可用的缓存配置')
  }
}

const loadSpecificCacheConfig = (storageType: string) => {
  // 提取纯存储类型名称（移除可能的会话标识）
  const pureStorageType = storageType.split(' ')[0] // 移除 " (user-xxx)" 部分

  let cached: any = null

  if (sessionId.value) {
    // 如果已连接，从当前会话加载
    cached = ConnectionCacheService.loadConnectionConfig(pureStorageType, sessionId.value)
  } else {
    // 如果未连接，从所有可用缓存中查找该存储类型的配置
    const allAvailableInfo = ConnectionCacheService.getAllAvailableCacheInfo()
    const targetCache = allAvailableInfo.find(cache => cache.storageType === pureStorageType)

    if (targetCache) {
      cached = targetCache.config
    }
  }

  if (cached) {
    // 更新存储类型
    selectedStorageType.value = cached.storageType

    // 重置并更新连接配置
    Object.assign(connectionConfig, {
      url: cached.storageType === 'local' ? '/tmp' : '',
      endpoint: '',
      bucket: '',
      access_key: '',
      secret_key: '',
      username: '',
      password: '',
    }, cached.connectionConfig)

    // 更新OSS配置
    Object.assign(ossConfig, cached.ossConfig)

    // 更新连接端点
    updateConnectionEndpoint()

    // 显示成功提示
    appStore.setGlobalError('')  // 清除可能存在的错误信息
    console.log(`已加载${ConnectionCacheService.getStorageTypeName(pureStorageType)}的缓存配置`)
  } else {
    console.log(`没有找到${ConnectionCacheService.getStorageTypeName(pureStorageType)}的缓存配置`)
  }
}

const saveCacheConfig = () => {
  if (isConnected.value && sessionId.value) {
    // 设置当前会话ID并保存缓存
    ConnectionCacheService.setCurrentSession(sessionId.value)
    ConnectionCacheService.saveConnectionConfig(
      selectedStorageType.value,
      connectionConfig,
      ossConfig,
      sessionId.value
    )
  }
}

const clearCache = () => {
  // 只清除当前用户会话的缓存
  ConnectionCacheService.clearCache(undefined, sessionId.value || undefined)
  updateCacheInfo()
}

const clearSpecificCache = (storageType: string) => {
  // 提取纯存储类型名称（移除可能的会话标识）
  const pureStorageType = storageType.split(' ')[0] // 移除 " (user-xxx)" 部分

  // 如果已连接，清除当前用户会话的特定存储类型缓存
  // 如果未连接，清除所有会话中该存储类型的缓存
  if (sessionId.value) {
    ConnectionCacheService.clearCache(pureStorageType, sessionId.value)
  } else {
    // 清除所有会话中该存储类型的缓存
    // 先获取所有可用缓存信息来确定需要清理的会话
    const allAvailableInfo = ConnectionCacheService.getAllAvailableCacheInfo()
    const targetCaches = allAvailableInfo.filter(cache => cache.storageType === pureStorageType)

    // 逐个清理
    targetCaches.forEach(cache => {
      const sessionIdFromConfig = cache.config.sessionId
      ConnectionCacheService.clearCache(pureStorageType, sessionIdFromConfig)
    })
  }

  updateCacheInfo()
}

const updateCacheInfo = () => {
  let cacheInfo: Array<{ storageType: string; lastUsed: string; config: any }> = []

  if (sessionId.value) {
    // 连接状态：显示当前用户会话的缓存信息
    cacheInfo = ConnectionCacheService.getAllCacheInfo(sessionId.value)
  } else {
    // 断开连接状态：显示所有可用的缓存信息（包括所有会话的缓存）
    cacheInfo = ConnectionCacheService.getAllAvailableCacheInfo()

    // 如果没有任何缓存，尝试获取默认缓存（兼容旧版本）
    if (cacheInfo.length === 0) {
      cacheInfo = ConnectionCacheService.getAllCacheInfo('default')
    }
  }

  allCacheInfo.value = cacheInfo
  console.log('更新缓存信息:', {
    sessionId: sessionId.value,
    cacheCount: cacheInfo.length,
    isConnected: !!sessionId.value
  })
}

// 连接存储
const connect = async () => {
  console.log('🔌 Connect clicked, current state:', {
    isConnected: isConnected.value,
    sessionId: sessionId.value,
    selectedStorageType: selectedStorageType.value,
    connectionConfig: connectionConfig
  })

  if (isConnected.value) {
    // 断开连接
    try {
      await fetch(`/api/storage/disconnect/${sessionId.value}`, {
        method: 'DELETE',
      })
      isConnected.value = false

      // 清理缓存服务的当前会话状态
      ConnectionCacheService.setCurrentSession(null)

      sessionId.value = ''
      files.value = []

      // 更新缓存信息显示（显示所有可用的缓存，包括默认和其他会话的）
      updateCacheInfo()
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
      await loadFiles('')
      saveCacheConfig() // 保存连接配置到缓存
      updateCacheInfo() // 更新缓存信息显示
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

// 加载文件列表（支持自动分页）
const loadFiles = async (path: string) => {
  if (!isConnected.value) return

  isLoading.value = true
  try {
    let allFiles: any[] = []
    let hasMore = true
    let marker: string | undefined = undefined

    // 循环加载所有页面
    while (hasMore) {
      const response = await fetch(`/api/storage/${sessionId.value}/list`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          session_id: sessionId.value,
          path: path || undefined,
          options: {
            page_size: 1000,
            sort_by: 'name',
            sort_order: 'asc',
            marker: marker,
          },
        }),
      })

      const data = await response.json()

      if (data.status === 'success') {
        allFiles = allFiles.concat(data.data.files)
        hasMore = data.data.has_more || false
        marker = data.data.next_marker
      } else {
        appStore.setGlobalError(`加载文件失败: ${data.message}`)
        break
      }

      // 安全检查，避免无限循环
      if (allFiles.length > 10000) {
        console.warn('文件数量超过10000个，停止加载')
        break
      }
    }

    files.value = allFiles
    currentPath.value = path
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

// 导航到指定路径
const navigateToPath = (path: string) => {
  loadFiles(path)
}

// 导航到上级目录
const navigateToParent = () => {
  if (currentPath.value) {
    const pathParts = currentPath.value.split('/').filter(Boolean)
    pathParts.pop() // 移除最后一个部分
    const parentPath = pathParts.join('/')
    loadFiles(parentPath)
  }
}

// 检测是否为压缩包文件
const isArchiveFile = (filename: string): boolean => {
  const archiveExtensions = ['.zip', '.tar', '.gz', '.bz2']
  const lowerFilename = filename.toLowerCase()
  return archiveExtensions.some(ext => lowerFilename.endsWith(ext))
}

// 判断是否为代码文件
const isCodeFile = (filename?: string): boolean => {
  if (!filename) return false
  const codeExtensions = ['.js', '.ts', '.jsx', '.tsx', '.vue', '.html', '.css', '.scss', '.sass',
    '.py', '.java', '.cpp', '.c', '.h', '.go', '.rs', '.php', '.rb', '.sh', '.sql', '.json',
    '.xml', '.yml', '.yaml', '.md', '.txt', '.log']
  const lowerFilename = filename.toLowerCase()
  return codeExtensions.some(ext => lowerFilename.endsWith(ext))
}

// 获取文件语言类型
const getFileLanguage = (filename?: string): string => {
  if (!filename) return 'text'
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  const languageMap: Record<string, string> = {
    js: 'javascript', ts: 'typescript', jsx: 'javascript', tsx: 'typescript',
    vue: 'vue', html: 'html', css: 'css', scss: 'scss', py: 'python',
    java: 'java', cpp: 'cpp', c: 'c', go: 'go', rs: 'rust',
    php: 'php', rb: 'ruby', sh: 'bash', sql: 'sql', json: 'json',
    xml: 'xml', yml: 'yaml', yaml: 'yaml', md: 'markdown'
  }
  return languageMap[ext] || 'text'
}

// 获取语法高亮的内容
const getHighlightedContent = (): string => {
  if (!fileContent.value || !previewFile.value?.filename) {
    return fileContent.value || ''
  }

  if (!isCodeFile(previewFile.value.filename)) {
    return fileContent.value
  }

  // 简单的语法高亮实现
  const language = getFileLanguage(previewFile.value.filename)
  return applySimpleSyntaxHighlight(fileContent.value, language)
}

// 简单语法高亮实现
const applySimpleSyntaxHighlight = (code: string, language: string): string => {
  if (language === 'json') {
    return code
      .replace(/(".*?")\s*:/g, '<span style="color: #0969da; font-weight: bold;">$1</span>:')
      .replace(/:\s*(".*?")/g, ': <span style="color: #0a3069;">$1</span>')
      .replace(/:\s*(true|false|null)/g, ': <span style="color: #8250df;">$1</span>')
      .replace(/:\s*(\d+\.?\d*)/g, ': <span style="color: #0969da;">$1</span>')
  }

  if (language === 'javascript' || language === 'typescript') {
    return code
      .replace(/\b(function|const|let|var|if|else|for|while|return|import|export|class|extends)\b/g, '<span style="color: #cf222e; font-weight: bold;">$1</span>')
      .replace(/('[^']*'|"[^"]*")/g, '<span style="color: #0a3069;">$1</span>')
      .replace(/\b(\d+\.?\d*)\b/g, '<span style="color: #0969da;">$1</span>')
      .replace(/(\/\/.*$)/gm, '<span style="color: #656d76; font-style: italic;">$1</span>')
  }

  if (language === 'python') {
    return code
      .replace(/\b(def|class|if|else|elif|for|while|return|import|from|try|except|with|as)\b/g, '<span style="color: #cf222e; font-weight: bold;">$1</span>')
      .replace(/('[^']*'|"[^"]*")/g, '<span style="color: #0a3069;">$1</span>')
      .replace(/\b(\d+\.?\d*)\b/g, '<span style="color: #0969da;">$1</span>')
      .replace(/(#.*$)/gm, '<span style="color: #656d76; font-style: italic;">$1</span>')
  }

  return code
}

// 处理文件点击
const handleFileClick = (file: any) => {
  if (file.type === 'directory') {
    const newPath = currentPath.value ? `${currentPath.value}/${file.basename}` : file.basename
    loadFiles(newPath)
  } else if (isArchiveFile(file.basename)) {
    // 浏览压缩包文件
    openArchiveBrowse(file)
  } else {
    // 预览文件
    openFilePreview(file)
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
      // 实际的下载逻辑需要后端支持
      console.log('Download started:', data.data.download_id)
    } else {
      appStore.setGlobalError(`下载失败: ${data.message}`)
    }
  } catch (error) {
    console.error('Download failed:', error)
    appStore.setGlobalError('下载失败')
  }
}

// 文件预览（支持大文件分块加载）
const openFilePreview = async (file: any) => {
  console.log('🔍 openFilePreview called with file:', file)
  previewFile.value = file
  isPreviewOpen.value = true
  isLoadingContent.value = true
  fileContent.value = ''
  fileLoadOffset.value = 0
  fileLoadedSize.value = 0
  fileTotalSize.value = parseInt(file.size || '0')

  console.log('🔍 File preview state:', {
    filename: file.filename,
    size: file.size,
    sessionId: sessionId.value,
    shouldUseVirtualized: shouldUseVirtualizedViewer.value
  })

  try {
    await loadFileChunk(file, 0, CHUNK_SIZE)
  } catch (error) {
    console.error('File preview failed:', error)
    appStore.setGlobalError('文件预览失败')
    isLoadingContent.value = false
  }
}

// 加载文件块
const loadFileChunk = async (file: any, offset: number, size: number) => {
  const filePath = currentPath.value ? `${currentPath.value}/${file.basename}` : file.basename

  console.log('📄 loadFileChunk called:', {
    filename: file.filename,
    basename: file.basename,
    filePath,
    offset,
    size,
    sessionId: sessionId.value,
    currentPath: currentPath.value
  })

  try {
    const response = await fetch(`/api/storage/${sessionId.value}/file/content`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        session_id: sessionId.value,
        path: filePath,
        start: offset > 0 ? offset : undefined,
        length: size > 0 ? size : undefined,
      }),
    })

    const data = await response.json()
    console.log('📡 Response:', { status: data.status, dataKeys: Object.keys(data.data || {}) })

    if (data.status === 'success') {
      // 将字节数组转换为文本
      const content = data.data.content
      const decoder = new TextDecoder('utf-8')
      const uint8Array = new Uint8Array(content)
      const chunkText = decoder.decode(uint8Array)

      console.log('📄 Content decoded:', {
        contentLength: content.length,
        textLength: chunkText.length,
        preview: chunkText.substring(0, 100) + (chunkText.length > 100 ? '...' : '')
      })

      // 第一次加载或者追加内容
      if (offset === 0) {
        fileContent.value = chunkText
      } else {
        fileContent.value += chunkText
      }

      fileLoadOffset.value = offset + chunkText.length
      fileLoadedSize.value += chunkText.length

      console.log('📄 Final state (before setting loading false):', {
        fileContentLength: fileContent.value.length,
        isLoadingContent: isLoadingContent.value,
        totalLoaded: fileLoadedSize.value,
        offset: offset
      })

      // 如果是初始加载，显示内容
      if (offset === 0) {
        console.log('📄 Setting isLoadingContent to false because offset === 0')
        isLoadingContent.value = false
        console.log('📄 After setting: isLoadingContent =', isLoadingContent.value)
      }
    } else {
      console.error('📄 API returned error:', data)
      appStore.setGlobalError(`文件预览失败: ${data.message}`)
      if (offset === 0) {
        isLoadingContent.value = false
      }
    }
  } catch (error) {
    console.error('Load file chunk failed:', error)
    appStore.setGlobalError('加载文件块失败')
    if (offset === 0) {
      isLoadingContent.value = false
    }
  }
}

// 加载更多内容（虚拟化查看器滚动到底部时调用）
const loadMoreContent = async () => {
  if (!previewFile.value || fileLoadedSize.value >= fileTotalSize.value) {
    return
  }

  console.log('Loading more content...', {
    loaded: fileLoadedSize.value,
    total: fileTotalSize.value,
    offset: fileLoadOffset.value
  })

  await loadFileChunk(previewFile.value, fileLoadOffset.value, CHUNK_SIZE)
}

const closeFilePreview = () => {
  isPreviewOpen.value = false
  previewFile.value = null
  fileContent.value = ''
}

// 打开压缩包浏览
const openArchiveBrowse = async (file: any) => {
  archiveFile.value = file
  isArchiveBrowseOpen.value = true
  isLoadingArchive.value = true
  archiveInfo.value = null
  archiveCurrentPath.value = ''

  try {
    const filePath = currentPath.value ? `${currentPath.value}/${file.basename}` : file.basename
    const response = await fetch(`/api/storage/${sessionId.value}/archive/info`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        file_path: filePath,
        max_entries: 1000,
      }),
    })

    const data = await response.json()
    if (data.status === 'success') {
      archiveInfo.value = data.data
    } else {
      appStore.setGlobalError(`无法读取压缩包: ${data.message}`)
    }
  } catch (error) {
    console.error('Archive browse failed:', error)
    appStore.setGlobalError('压缩包浏览失败')
  } finally {
    isLoadingArchive.value = false
  }
}

// 关闭压缩包浏览
const closeArchiveBrowse = () => {
  isArchiveBrowseOpen.value = false
  archiveFile.value = null
  archiveInfo.value = null
  archiveCurrentPath.value = ''
}

// 获取当前路径下的文件列表（用于Archive浏览）
const getArchiveFilesInPath = () => {
  if (!archiveInfo.value) return []

  const currentPathParts = archiveCurrentPath.value ? archiveCurrentPath.value.split('/').filter(Boolean) : []
  const files: any[] = []
  const directories = new Set<string>()

  archiveInfo.value.entries.forEach((entry: any) => {
    const pathParts = entry.path.split('/').filter(Boolean)

    // 如果文件不在当前路径下，跳过
    if (pathParts.length <= currentPathParts.length) return

    // 检查路径前缀是否匹配
    for (let i = 0; i < currentPathParts.length; i++) {
      if (pathParts[i] !== currentPathParts[i]) return
    }

    const nextPart = pathParts[currentPathParts.length]

    // 如果这是直接子文件（没有更深层的路径）
    if (pathParts.length === currentPathParts.length + 1) {
      files.push({
        name: nextPart,
        type: 'file',
        size: entry.size,
        path: entry.path,
        isDirectory: false
      })
    } else {
      // 这是一个目录
      directories.add(nextPart)
    }
  })

  // 添加目录
  directories.forEach(dirName => {
    files.unshift({
      name: dirName,
      type: 'directory',
      size: 0,
      path: archiveCurrentPath.value ? `${archiveCurrentPath.value}/${dirName}` : dirName,
      isDirectory: true
    })
  })

  return files.sort((a, b) => {
    if (a.isDirectory && !b.isDirectory) return -1
    if (!a.isDirectory && b.isDirectory) return 1
    return a.name.localeCompare(b.name)
  })
}

// 处理Archive文件点击
const handleArchiveFileClick = (archiveFileItem: any) => {
  if (archiveFileItem.isDirectory) {
    archiveCurrentPath.value = archiveFileItem.path
  } else {
    openArchiveFilePreview(archiveFileItem)
  }
}

// 导航到Archive父目录
const navigateToArchiveParent = () => {
  if (archiveCurrentPath.value) {
    const parts = archiveCurrentPath.value.split('/').filter(Boolean)
    parts.pop()
    archiveCurrentPath.value = parts.join('/')
  }
}

// 预览Archive中的文件
const openArchiveFilePreview = async (archiveFileItem: any) => {
  isLoadingContent.value = true
  fileContent.value = ''

  // 显示加载提示
  console.log(`开始预览ZIP文件中的: ${archiveFileItem.path}`)

  try {
    const archiveFilePath = currentPath.value ? `${currentPath.value}/${archiveFile.value.basename}` : archiveFile.value.basename

    // 创建超时控制器
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 30000) // 30秒超时

    const response = await fetch(`/api/storage/${sessionId.value}/archive/file`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        archive_path: archiveFilePath,
        file_path: archiveFileItem.path,
        max_size: 10 * 1024 * 1024, // 10MB 限制
      }),
      signal: controller.signal, // 添加超时控制
    })

    clearTimeout(timeoutId) // 清除超时

    const data = await response.json()
    if (data.status === 'success') {
      // 设置预览文件信息
      previewFile.value = {
        filename: archiveFileItem.name,
        size: archiveFileItem.size.toString(),
        path: archiveFileItem.path,
        isArchiveFile: true
      }

      // 将字节数组转换为文本
      const content = data.data.content
      const decoder = new TextDecoder('utf-8')
      const uint8Array = new Uint8Array(content)
      fileContent.value = decoder.decode(uint8Array)

      // 关闭Archive浏览，打开文件预览
      closeArchiveBrowse()
      isPreviewOpen.value = true

      console.log(`成功预览文件: ${archiveFileItem.path}`)
    } else {
      console.error('预览失败:', data.message)
      appStore.setGlobalError(`无法预览文件: ${data.message}`)
    }
  } catch (error) {
    console.error('Archive file preview failed:', error)

    if (error.name === 'AbortError') {
      appStore.setGlobalError('文件预览超时（30秒），ZIP文件可能太大。请尝试预览更小的文件。')
    } else if (error.message.includes('Failed to fetch')) {
      appStore.setGlobalError('网络请求失败，请检查网络连接')
    } else {
      appStore.setGlobalError('文件预览失败')
    }
  } finally {
    isLoadingContent.value = false
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

// 检查现有活跃会话
const checkExistingSessions = async () => {
  try {
    const response = await fetch('/api/storage/sessions')
    const data = await response.json()

    if (data.status === 'success' && data.data.length > 0) {
      // 有活跃会话，获取第一个会话的详细信息
      const existingSessionId = data.data[0]
      const sessionResponse = await fetch(`/api/storage/sessions/${existingSessionId}`)
      const sessionData = await sessionResponse.json()

      if (sessionData.status === 'success' && sessionData.data.connected) {
        // 恢复前端状态
        isConnected.value = true
        sessionId.value = sessionData.data.session_id

        // 尝试加载文件列表
        await loadFiles('')

        console.log('已恢复活跃的存储会话:', existingSessionId)
      }
    }
  } catch (error) {
    console.log('检查现有会话失败:', error)
  }
}

// 组件挂载时更新缓存信息并检查现有会话
onMounted(async () => {
  updateCacheInfo()
  await checkExistingSessions()
})
</script>

<style scoped>
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 4px;
  border: 2px solid transparent;
  background-clip: content-box;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.7);
}

.dark .custom-scrollbar {
  scrollbar-color: rgba(75, 85, 99, 0.5) transparent;
}

.dark .custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(75, 85, 99, 0.5);
}

.dark .custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(75, 85, 99, 0.7);
}
</style>