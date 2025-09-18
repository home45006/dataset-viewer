// 简单的存储管理器，用于 Vue Web 版本
// 直接通过 HTTP API 与后端通信

export class StorageManager {
  // Vue Web 版本不需要复杂的存储客户端
  // 所有操作通过 HTTP API 完成

  static async connect(config: any) {
    // 实际连接通过 Vue 组件中的 fetch API 完成
    console.log('StorageManager.connect called with config:', config)
    return { success: true }
  }

  static async listDirectory(path: string) {
    console.log('StorageManager.listDirectory called with path:', path)
    return { files: [] }
  }
}

export default StorageManager