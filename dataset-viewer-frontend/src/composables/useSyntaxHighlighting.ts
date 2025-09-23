import { ref, computed, onMounted } from 'vue'
import { preloadHighlighters } from '../utils/syntaxHighlighter'

// 全局语法高亮开关
const globalEnabled = ref(true)

export function useSyntaxHighlighting() {
  const enabled = computed({
    get: () => globalEnabled.value,
    set: (value: boolean) => {
      globalEnabled.value = value
      // 保存到本地存储
      localStorage.setItem('syntax-highlighting-enabled', JSON.stringify(value))
    }
  })

  // 从本地存储恢复设置
  onMounted(() => {
    const saved = localStorage.getItem('syntax-highlighting-enabled')
    if (saved !== null) {
      try {
        globalEnabled.value = JSON.parse(saved)
      } catch (error) {
        console.warn('Failed to parse syntax highlighting setting:', error)
      }
    }

    // 预加载高亮器
    if (globalEnabled.value) {
      preloadHighlighters().catch(error => {
        console.warn('Failed to preload syntax highlighters:', error)
      })
    }
  })

  return {
    enabled
  }
}