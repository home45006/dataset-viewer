import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAppStore = defineStore('app', () => {
  // 状态
  const isLoading = ref(false)
  const globalError = ref<string | null>(null)
  const theme = ref<'light' | 'dark' | 'auto'>('auto')

  // 计算属性
  const isDark = computed(() => {
    if (theme.value === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return theme.value === 'dark'
  })

  // 方法
  const setLoading = (loading: boolean) => {
    isLoading.value = loading
  }

  const setGlobalError = (error: string | null) => {
    globalError.value = error
  }

  const clearGlobalError = () => {
    globalError.value = null
  }

  const setTheme = (newTheme: 'light' | 'dark' | 'auto') => {
    theme.value = newTheme
    localStorage.setItem('theme', newTheme)
    
    // 更新 DOM
    const html = document.documentElement
    if (newTheme === 'dark') {
      html.classList.add('dark')
    } else if (newTheme === 'light') {
      html.classList.remove('dark')
    } else {
      // auto mode
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        html.classList.add('dark')
      } else {
        html.classList.remove('dark')
      }
    }
  }

  const toggleTheme = () => {
    if (theme.value === 'light') {
      setTheme('dark')
    } else if (theme.value === 'dark') {
      setTheme('auto')
    } else {
      setTheme('light')
    }
  }

  // 初始化主题
  const initTheme = () => {
    const saved = localStorage.getItem('theme') as 'light' | 'dark' | 'auto' | null
    if (saved) {
      setTheme(saved)
    }
  }

  return {
    // 状态
    isLoading,
    globalError,
    theme,
    
    // 计算属性
    isDark,
    
    // 方法
    setLoading,
    setGlobalError,
    clearGlobalError,
    setTheme,
    toggleTheme,
    initTheme,
  }
})