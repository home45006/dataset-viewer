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
}

export interface MultiStorageCache {
  [storageType: string]: StorageConnectionCache
}

export class ConnectionCacheService {
  private static readonly CACHE_KEY = 'dataset-viewer-connection-cache-v2'
  private static readonly MAX_CACHE_AGE = 30 * 24 * 60 * 60 * 1000 // 30天

  /**
   * 保存连接配置到缓存
   */
  static saveConnectionConfig(
    storageType: string,
    connectionConfig: ConnectionConfig,
    ossConfig: OSSConfig
  ): void {
    try {
      // 获取现有缓存
      const existingCache = this.loadAllConfigs()

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
      }

      // 更新缓存数据
      existingCache[storageType] = currentCacheData

      localStorage.setItem(this.CACHE_KEY, JSON.stringify(existingCache))
      console.log(`${this.getStorageTypeName(storageType)} 配置已保存到缓存`)
    } catch (error) {
      console.warn('保存连接配置到缓存失败:', error)
    }
  }

  /**
   * 加载所有缓存配置
   */
  static loadAllConfigs(): MultiStorageCache {
    try {
      const cached = localStorage.getItem(this.CACHE_KEY)
      if (!cached) {
        return {}
      }

      const cacheData: MultiStorageCache = JSON.parse(cached)
      const now = Date.now()
      const validCache: MultiStorageCache = {}

      // 过滤掉过期的缓存
      for (const [storageType, config] of Object.entries(cacheData)) {
        if (now - config.lastUsed <= this.MAX_CACHE_AGE) {
          validCache[storageType] = config
        }
      }

      // 如果有配置被过滤掉，更新缓存
      if (Object.keys(validCache).length !== Object.keys(cacheData).length) {
        localStorage.setItem(this.CACHE_KEY, JSON.stringify(validCache))
      }

      return validCache
    } catch (error) {
      console.warn('加载缓存配置失败:', error)
      this.clearCache() // 清除损坏的缓存
      return {}
    }
  }

  /**
   * 从缓存加载特定存储类型的连接配置
   */
  static loadConnectionConfig(storageType?: string): StorageConnectionCache | null {
    try {
      const allConfigs = this.loadAllConfigs()

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

        console.log('从缓存加载连接配置:', this.getStorageTypeName(mostRecent.storageType))
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
  static clearCache(storageType?: string): void {
    try {
      if (storageType) {
        // 清除特定存储类型的缓存
        const allConfigs = this.loadAllConfigs()
        delete allConfigs[storageType]
        localStorage.setItem(this.CACHE_KEY, JSON.stringify(allConfigs))
        console.log(`${this.getStorageTypeName(storageType)} 配置缓存已清除`)
      } else {
        // 清除所有缓存
        localStorage.removeItem(this.CACHE_KEY)
        console.log('所有连接配置缓存已清除')
      }
    } catch (error) {
      console.warn('清除缓存失败:', error)
    }
  }

  /**
   * 检查是否有缓存
   */
  static hasCache(storageType?: string): boolean {
    try {
      const allConfigs = this.loadAllConfigs()

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
  static getCacheInfo(): { hasCache: boolean; lastUsed?: string; storageType?: string } {
    try {
      const allConfigs = this.loadAllConfigs()
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
   * 获取所有缓存的存储类型列表
   */
  static getCachedStorageTypes(): string[] {
    try {
      const allConfigs = this.loadAllConfigs()
      return Object.keys(allConfigs).sort()
    } catch {
      return []
    }
  }

  /**
   * 获取所有缓存配置的信息
   */
  static getAllCacheInfo(): Array<{ storageType: string; lastUsed: string; config: StorageConnectionCache }> {
    try {
      const allConfigs = this.loadAllConfigs()
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