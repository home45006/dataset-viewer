// 简单的存储客户端，用于 Vue Web 版本
// 直接通过 HTTP API 与后端通信

export class StorageClient {
  constructor(public type: string) {}

  async connect(config: any): Promise<boolean> {
    // 实际连接通过 Vue 组件中的 fetch API 完成
    console.log('StorageClient.connect called with config:', config)
    return true
  }

  async listDirectory(path: string): Promise<any[]> {
    console.log('StorageClient.listDirectory called with path:', path)
    return []
  }

  async getFileContent(path: string): Promise<string> {
    console.log('StorageClient.getFileContent called with path:', path)
    return ''
  }

  disconnect(): void {
    console.log('StorageClient.disconnect called')
  }

  isConnected(): boolean {
    return false
  }
}

export default StorageClient