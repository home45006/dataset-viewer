import { ArchiveInfo, FilePreview } from '../types';

// 检测当前是否在 Web 环境中
const isWebEnvironment = typeof window !== 'undefined' && !(window as any).__TAURI_IPC__;

// Web 环境下的压缩包服务
export class WebCompressionService {
  private static baseUrl = 'http://localhost:8080/api';
  private static sessionId: string | null = null;

  /**
   * 获取当前会话 ID（从后端 API 或本地存储获取）
   */
  private static async getSessionId(): Promise<string> {
    if (!this.sessionId) {
      // 首先尝试从本地存储获取 session ID
      const stored = localStorage.getItem('current-session-id');
      console.log('WebCompressionService: Checking localStorage for session ID:', stored);
      if (stored) {
        this.sessionId = stored;
        console.log('WebCompressionService: Using session ID from localStorage:', this.sessionId);
        return this.sessionId;
      }

      // 如果本地存储没有，尝试从后端 API 获取当前会话
      try {
        const response = await fetch(`${this.baseUrl}/sessions/current`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (response.ok) {
          const result = await response.json();
          if (result.status === 'success' && result.data?.session_id) {
            this.sessionId = result.data.session_id;
            localStorage.setItem('current-session-id', this.sessionId);
            return this.sessionId;
          }
        }
      } catch (error) {
        console.warn('Failed to get current session from API:', error);
      }

      throw new Error('No active storage session found. Please ensure you are connected to a storage service.');
    }
    return this.sessionId;
  }

  /**
   * 设置会话 ID
   */
  static setSessionId(sessionId: string) {
    this.sessionId = sessionId;
    localStorage.setItem('current-session-id', sessionId);
  }

  /**
   * 分析压缩文件结构
   */
  static async analyzeArchive(
    _url: string,
    filename: string,
    maxSize?: number
  ): Promise<ArchiveInfo> {
    if (!isWebEnvironment) {
      throw new Error('WebCompressionService can only be used in web environment');
    }

    try {
      const sessionId = await this.getSessionId();
      console.log('WebCompressionService.analyzeArchive: Using session ID:', sessionId);

      const response = await fetch(`${this.baseUrl}/storage/${sessionId}/archive/info`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          file_path: filename, // 传递相对路径
          max_entries: maxSize || 1000,
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`HTTP ${response.status}: ${errorText}`);
      }

      const result = await response.json();

      if (result.status === 'error') {
        throw new Error(result.message || result.error || 'Unknown error');
      }

      return result.data;
    } catch (error) {
      console.error('WebCompressionService.analyzeArchive failed:', error);
      throw error;
    }
  }

  /**
   * 从压缩文件中提取文件预览
   */
  static async extractFilePreview(
    _url: string,
    filename: string,
    entryPath: string,
    maxPreviewSize?: number
  ): Promise<FilePreview> {
    if (!isWebEnvironment) {
      throw new Error('WebCompressionService can only be used in web environment');
    }

    try {
      const sessionId = await this.getSessionId();
      console.log('WebCompressionService.extractFilePreview: Using session ID:', sessionId);

      const response = await fetch(`${this.baseUrl}/storage/${sessionId}/archive/file`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          archive_path: filename,
          file_path: entryPath,
          max_size: maxPreviewSize || null,
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`HTTP ${response.status}: ${errorText}`);
      }

      const result = await response.json();

      if (result.status === 'error') {
        throw new Error(result.message || result.error || 'Unknown error');
      }

      const data = result.data;

      // 将字节数组转换为 Uint8Array
      const content = new Uint8Array(data.content);

      return {
        content,
        is_truncated: data.is_truncated,
        total_size: data.total_size.toString(),
        preview_size: data.preview_size,
      };
    } catch (error) {
      console.error('WebCompressionService.extractFilePreview failed:', error);
      throw error;
    }
  }
}

/**
 * 统一的压缩服务，根据环境自动选择合适的实现
 */
export class UnifiedCompressionService {
  /**
   * 分析压缩文件结构
   */
  static async analyzeArchive(
    _url: string,
    filename: string,
    maxSize?: number
  ): Promise<ArchiveInfo> {
    if (isWebEnvironment) {
      return WebCompressionService.analyzeArchive(url, filename, maxSize);
    } else {
      // 在 Tauri 环境中，回退到原来的 CompressionService
      const { CompressionService } = await import('./compression');
      return CompressionService.analyzeArchive(url, filename, maxSize);
    }
  }

  /**
   * 从压缩文件中提取文件预览
   */
  static async extractFilePreview(
    _url: string,
    filename: string,
    entryPath: string,
    maxPreviewSize?: number
  ): Promise<FilePreview> {
    if (isWebEnvironment) {
      return WebCompressionService.extractFilePreview(url, filename, entryPath, maxPreviewSize);
    } else {
      // 在 Tauri 环境中，回退到原来的 CompressionService
      const { CompressionService } = await import('./compression');
      return CompressionService.extractFilePreview(url, filename, entryPath, maxPreviewSize);
    }
  }

  /**
   * 设置会话 ID（仅在 Web 环境中需要）
   */
  static setSessionId(sessionId: string) {
    if (isWebEnvironment) {
      WebCompressionService.setSessionId(sessionId);
    }
  }

  /**
   * 清除会话 ID（用于连接切换时）
   */
  static clearSession() {
    if (isWebEnvironment) {
      WebCompressionService.sessionId = null;
      localStorage.removeItem('current-session-id');
    }
  }

  /**
   * 调试帮助器：手动设置会话 ID（暴露到全局以便在浏览器控制台中使用）
   */
  static debugSetSessionId(sessionId: string) {
    if (isWebEnvironment) {
      console.log('Manually setting session ID:', sessionId);
      this.setSessionId(sessionId);
      // 同时暴露到全局对象，方便调试
      (window as any).debugCompressionSessionId = sessionId;
    }
  }

  /**
   * 检查当前是否在 Web 环境中
   */
  static isWebEnvironment(): boolean {
    return isWebEnvironment;
  }

  /**
   * 从后端 API 获取当前活跃的会话 ID（如果有）
   */
  static async initializeSession(): Promise<void> {
    if (isWebEnvironment) {
      try {
        // 调用一个公共方法来初始化会话
        const hasSession = localStorage.getItem('current-session-id');
        if (!hasSession) {
          console.warn('No session found in localStorage, session will be initialized on first API call');
        }
      } catch (error) {
        console.warn('Failed to initialize web session:', error);
        // 不抛出错误，允许应用继续运行
      }
    }
  }
}

// 在 Web 环境中，将 UnifiedCompressionService 暴露到全局对象用于调试
if (isWebEnvironment) {
  (window as any).UnifiedCompressionService = UnifiedCompressionService;
  console.log('Debug: UnifiedCompressionService available globally for debugging');
}