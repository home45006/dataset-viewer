/**
 * 存储连接表单缓存服务
 * 用于记忆用户上次填写的连接参数
 */

export interface ConnectionConfig {
  url: string
  endpoint: string
  bucket: string
  access_key: string
  secret_key: string
  username: string
  password: string
}

export interface OSSConfig {
  platform: string
  region: string
  customEndpoint: string
}

export interface StorageConnectionCache {
  storageType: string
  connectionConfig: ConnectionConfig
  ossConfig: OSSConfig
  lastUsed: number
  sessionId?: string // 添加会话ID标识
}

export interface MultiStorageCache {
  [storageType: string]: StorageConnectionCache
}

// 全局缓存结构，包含所有用户的缓存
export interface GlobalCacheStructure {
  [cacheKey: string]: MultiStorageCache // cacheKey 格式: "user-{sessionId}" 或兼容旧版本的默认键
}

export class ConnectionCacheService {
  private static readonly CACHE_KEY = 'dataset-viewer-connection-cache-v2'
  private static readonly MAX_CACHE_AGE = 30 * 24 * 60 * 60 * 1000 // 30天

  // 当前用户会话ID（可以通过setCurrentSession设置）
  private static currentSessionId: string | null = null

  /**
   * 设置当前用户的会话ID
   */
  static setCurrentSession(sessionId: string | null): void {
    this.currentSessionId = sessionId
  }

  /**
   * 获取当前用户的会话ID
   */
  static getCurrentSession(): string | null {
    return this.currentSessionId
  }

  /**
   * 获取用户级缓存键
   */
  private static getUserCacheKey(sessionId?: string): string {
    const session = sessionId || this.currentSessionId
    // 如果传入'default'，直接使用，否则根据sessionId生成用户键
    if (sessionId === 'default') {
      return 'default'
    }
    return session ? `user-${session}` : 'default' // 兼容旧版本
  }

  /**
   * 保存连接配置到缓存
   */
  static saveConnectionConfig(
    storageType: string,
    connectionConfig: ConnectionConfig,
    ossConfig: OSSConfig,
    sessionId?: string
  ): void {
    try {
      // 获取全局缓存结构
      const globalCache = this.loadGlobalCache()
      const userCacheKey = this.getUserCacheKey(sessionId)

      // 获取当前用户的缓存，如果不存在则创建
      if (!globalCache[userCacheKey]) {
        globalCache[userCacheKey] = {}
      }

      // 创建当前存储类型的缓存对象，敏感信息（密码）不缓存
      const currentCacheData: StorageConnectionCache = {
        storageType,
        connectionConfig: {
          ...connectionConfig,
          // 不缓存敏感信息
          password: '',
          secret_key: '',
        },
        ossConfig: { ...ossConfig },
        lastUsed: Date.now(),
        sessionId: sessionId || this.currentSessionId || undefined,
      }

      // 更新用户缓存数据
      globalCache[userCacheKey][storageType] = currentCacheData

      // 清理同存储类型的旧缓存（其他用户的）
      this.cleanupOldStorageTypeCaches(globalCache, storageType, userCacheKey)

      localStorage.setItem(this.CACHE_KEY, JSON.stringify(globalCache))
      console.log(`${this.getStorageTypeName(storageType)} 配置已保存到用户缓存 (${userCacheKey})`)
    } catch (error) {
      console.warn('保存连接配置到缓存失败:', error)
    }
  }

  /**
   * 加载全局缓存结构
   */
  private static loadGlobalCache(): GlobalCacheStructure {
    try {
      const cached = localStorage.getItem(this.CACHE_KEY)
      if (!cached) {
        return {}
      }

      const cacheData = JSON.parse(cached)

      // 兼容旧版本：如果是旧的MultiStorageCache结构，转换为新结构
      if (cacheData && typeof cacheData === 'object' &&
          Object.values(cacheData).some(item =>
            item && typeof item === 'object' && 'storageType' in item)) {
        console.log('检测到旧版本缓存格式，正在迁移...')
        return { 'default': cacheData as MultiStorageCache }
      }

      return cacheData as GlobalCacheStructure
    } catch (error) {
      console.warn('加载全局缓存失败:', error)
      return {}
    }
  }

  /**
   * 加载所有缓存配置（仅当前用户的）
   */
  static loadAllConfigs(sessionId?: string): MultiStorageCache {
    try {
      const globalCache = this.loadGlobalCache()
      const userCacheKey = this.getUserCacheKey(sessionId)
      const userCache = globalCache[userCacheKey] || {}

      const now = Date.now()
      const validCache: MultiStorageCache = {}

      // 过滤掉过期的缓存
      for (const [storageType, config] of Object.entries(userCache)) {
        if (now - config.lastUsed <= this.MAX_CACHE_AGE) {
          validCache[storageType] = config
        }
      }

      // 如果有配置被过滤掉，更新缓存
      if (Object.keys(validCache).length !== Object.keys(userCache).length) {
        globalCache[userCacheKey] = validCache
        localStorage.setItem(this.CACHE_KEY, JSON.stringify(globalCache))
      }

      return validCache
    } catch (error) {
      console.warn('加载缓存配置失败:', error)
      this.clearCache(undefined, sessionId) // 清除损坏的缓存
      return {}
    }
  }

  /**
   * 从缓存加载特定存储类型的连接配置
   */
  static loadConnectionConfig(storageType?: string, sessionId?: string): StorageConnectionCache | null {
    try {
      const allConfigs = this.loadAllConfigs(sessionId)

      if (storageType) {
        // 加载指定存储类型的配置
        return allConfigs[storageType] || null
      } else {
        // 返回最近使用的配置（兼容旧版本）
        const configs = Object.values(allConfigs)
        if (configs.length === 0) return null

        // 找到最近使用的配置
        const mostRecent = configs.reduce((latest, current) =>
          current.lastUsed > latest.lastUsed ? current : latest
        )

        console.log('从用户缓存加载连接配置:', this.getStorageTypeName(mostRecent.storageType))
        return mostRecent
      }
    } catch (error) {
      console.warn('从缓存加载连接配置失败:', error)
      return null
    }
  }

  /**
   * 清除缓存
   */
  static clearCache(storageType?: string, sessionId?: string): void {
    try {
      const globalCache = this.loadGlobalCache()
      const userCacheKey = this.getUserCacheKey(sessionId)

      if (storageType) {
        // 清除特定存储类型的缓存
        if (globalCache[userCacheKey]) {
          delete globalCache[userCacheKey][storageType]
          localStorage.setItem(this.CACHE_KEY, JSON.stringify(globalCache))
        }
        console.log(`${this.getStorageTypeName(storageType)} 用户配置缓存已清除`)
      } else {
        if (sessionId || this.currentSessionId) {
          // 清除当前用户的所有缓存
          delete globalCache[userCacheKey]
          localStorage.setItem(this.CACHE_KEY, JSON.stringify(globalCache))
          console.log(`用户 ${userCacheKey} 的所有连接配置缓存已清除`)
        } else {
          // 清除所有缓存（全局）
          localStorage.removeItem(this.CACHE_KEY)
          console.log('所有连接配置缓存已清除')
        }
      }
    } catch (error) {
      console.warn('清除缓存失败:', error)
    }
  }

  /**
   * 检查是否有缓存
   */
  static hasCache(storageType?: string, sessionId?: string): boolean {
    try {
      const allConfigs = this.loadAllConfigs(sessionId)

      if (storageType) {
        return storageType in allConfigs
      } else {
        return Object.keys(allConfigs).length > 0
      }
    } catch {
      return false
    }
  }

  /**
   * 获取缓存信息（用于显示）
   */
  static getCacheInfo(sessionId?: string): { hasCache: boolean; lastUsed?: string; storageType?: string } {
    try {
      const allConfigs = this.loadAllConfigs(sessionId)
      const configs = Object.values(allConfigs)

      if (configs.length === 0) {
        return { hasCache: false }
      }

      // 返回最近使用的配置信息
      const mostRecent = configs.reduce((latest, current) =>
        current.lastUsed > latest.lastUsed ? current : latest
      )

      return {
        hasCache: true,
        lastUsed: new Date(mostRecent.lastUsed).toLocaleString('zh-CN'),
        storageType: mostRecent.storageType,
      }
    } catch {
      return { hasCache: false }
    }
  }

  /**
   * 获取所有缓存的存储类型列表（仅当前用户的）
   */
  static getCachedStorageTypes(sessionId?: string): string[] {
    try {
      const allConfigs = this.loadAllConfigs(sessionId)
      return Object.keys(allConfigs).sort()
    } catch {
      return []
    }
  }

  /**
   * 获取所有缓存配置的信息（仅当前用户的）
   */
  static getAllCacheInfo(sessionId?: string): Array<{ storageType: string; lastUsed: string; config: StorageConnectionCache }> {
    try {
      const allConfigs = this.loadAllConfigs(sessionId)
      return Object.entries(allConfigs)
        .map(([storageType, config]) => ({
          storageType,
          lastUsed: new Date(config.lastUsed).toLocaleString('zh-CN'),
          config,
        }))
        .sort((a, b) => b.config.lastUsed - a.config.lastUsed) // 按最近使用时间排序
    } catch {
      return []
    }
  }

  /**
   * 获取所有可用的缓存信息（包括所有会话的缓存，用于断开连接时显示）
   */
  static getAllAvailableCacheInfo(): Array<{ storageType: string; lastUsed: string; config: StorageConnectionCache }> {
    try {
      const globalCache = this.loadGlobalCache()
      const storageTypeMap = new Map<string, { lastUsed: number; config: StorageConnectionCache }>()

      // 遍历所有用户的缓存，只保留每个存储类型的最新记录
      for (const [userKey, userCache] of Object.entries(globalCache)) {
        const now = Date.now()

        // 过滤掉过期的缓存
        for (const [storageType, config] of Object.entries(userCache)) {
          if (now - config.lastUsed <= this.MAX_CACHE_AGE) {
            const existing = storageTypeMap.get(storageType)

            // 如果没有现有记录，或者当前记录更新，则保存
            if (!existing || config.lastUsed > existing.lastUsed) {
              storageTypeMap.set(storageType, {
                lastUsed: config.lastUsed,
                config: {
                  ...config,
                  storageType // 保持原始存储类型，不添加会话标识
                }
              })
            }
          }
        }
      }

      // 转换为数组格式
      const allCacheInfo: Array<{ storageType: string; lastUsed: string; config: StorageConnectionCache }> = []
      for (const [storageType, { config }] of storageTypeMap.entries()) {
        allCacheInfo.push({
          storageType,
          lastUsed: new Date(config.lastUsed).toLocaleString('zh-CN'),
          config,
        })
      }

      return allCacheInfo.sort((a, b) => b.config.lastUsed - a.config.lastUsed)
    } catch {
      return []
    }
  }

  /**
   * 清理同存储类型的旧缓存，只保留最新的
   */
  private static cleanupOldStorageTypeCaches(
    globalCache: GlobalCacheStructure,
    storageType: string,
    currentUserKey: string
  ): void {
    try {
      const storageTypeCaches: Array<{ userKey: string; cache: StorageConnectionCache }> = []

      // 收集所有同存储类型的缓存
      for (const [userKey, userCache] of Object.entries(globalCache)) {
        if (userCache[storageType]) {
          storageTypeCaches.push({
            userKey,
            cache: userCache[storageType]
          })
        }
      }

      // 如果只有当前用户有这个存储类型的缓存，无需清理
      if (storageTypeCaches.length <= 1) {
        return
      }

      // 按最后使用时间排序，最新的在前
      storageTypeCaches.sort((a, b) => b.cache.lastUsed - a.cache.lastUsed)

      // 保留最新的（当前用户的应该是最新的），删除其他用户的旧缓存
      const toDelete = storageTypeCaches.slice(1)
      toDelete.forEach(({ userKey }) => {
        if (userKey !== currentUserKey) {
          delete globalCache[userKey][storageType]
          // 如果该用户没有其他缓存了，删除整个用户键
          if (Object.keys(globalCache[userKey]).length === 0) {
            delete globalCache[userKey]
          }
        }
      })

      if (toDelete.length > 0) {
        console.log(`已清理 ${toDelete.length} 个旧的 ${this.getStorageTypeName(storageType)} 缓存记录`)
      }
    } catch (error) {
      console.warn('清理旧缓存失败:', error)
    }
  }

  /**
   * 获取存储类型的显示名称
   */
  static getStorageTypeName(type: string): string {
    const names: Record<string, string> = {
      local: '本地文件',
      oss: '对象存储 (OSS/S3)',
      webdav: 'WebDAV',
      ssh: 'SSH/SFTP',
      smb: 'SMB/CIFS',
      huggingface: 'HuggingFace Hub',
    }
    return names[type] || type
  }
}