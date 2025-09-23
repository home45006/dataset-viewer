<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
    @click="$emit('close')"
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-lg max-w-6xl w-full max-h-[90vh] flex flex-col shadow-2xl"
      @click.stop
    >
      <!-- 头部 -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center space-x-3">
          <svg class="w-6 h-6 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"/>
          </svg>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              Markdown 预览
            </h3>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              {{ fileName || '未命名文件' }}
            </p>
          </div>
        </div>
        <button
          @click="$emit('close')"
          class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
          <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-hidden flex">
        <!-- 原始内容 -->
        <div class="w-1/2 border-r border-gray-200 dark:border-gray-700 flex flex-col">
          <div class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">原始 Markdown</h4>
          </div>
          <div class="flex-1 overflow-auto p-4">
            <pre class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap font-mono">{{ content }}</pre>
          </div>
        </div>

        <!-- 渲染预览 -->
        <div class="w-1/2 flex flex-col">
          <div class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">渲染预览</h4>
          </div>
          <div class="flex-1 overflow-auto p-4">
            <div
              v-if="renderedHtml"
              class="prose prose-sm dark:prose-invert max-w-none"
              v-html="renderedHtml"
            />
            <div v-else-if="isLoading" class="flex items-center justify-center h-32">
              <div class="flex items-center space-x-2 text-gray-500">
                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-500"></div>
                <span>渲染中...</span>
              </div>
            </div>
            <div v-else class="text-gray-500 text-center py-8">
              渲染失败
            </div>
          </div>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
        <div class="flex items-center justify-between">
          <div class="text-sm text-gray-500 dark:text-gray-400">
            字符数: {{ content.length.toLocaleString() }}
          </div>
          <div class="flex space-x-2">
            <button
              @click="copyToClipboard"
              class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
            >
              复制 Markdown
            </button>
            <button
              @click="$emit('close')"
              class="px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
            >
              关闭
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'

interface Props {
  isOpen: boolean
  content: string
  fileName?: string
}

const props = withDefaults(defineProps<Props>(), {
  fileName: ''
})

defineEmits<{
  close: []
}>()

const renderedHtml = ref('')
const isLoading = ref(false)

// 配置 marked
marked.setOptions({
  breaks: true,
  gfm: true,
  headerIds: false,
  mangle: false
})

// 渲染 Markdown
const renderMarkdown = async (content: string) => {
  if (!content.trim()) {
    renderedHtml.value = ''
    return
  }

  isLoading.value = true

  try {
    await nextTick()
    const html = marked(content)
    renderedHtml.value = DOMPurify.sanitize(html)
  } catch (error) {
    console.error('Markdown rendering error:', error)
    renderedHtml.value = ''
  } finally {
    isLoading.value = false
  }
}

// 监听内容变化
watch(() => props.content, (newContent) => {
  if (props.isOpen) {
    renderMarkdown(newContent)
  }
}, { immediate: false })

// 监听弹窗打开
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    renderMarkdown(props.content)
  }
})

// 复制到剪贴板
const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(props.content)
    // 这里可以添加成功提示
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = props.content
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
  }
}
</script>

<style scoped>
/* 自定义滚动条 */
.overflow-auto::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.overflow-auto::-webkit-scrollbar-track {
  @apply bg-gray-100 dark:bg-gray-800;
}

.overflow-auto::-webkit-scrollbar-thumb {
  @apply bg-gray-300 dark:bg-gray-600 rounded;
}

.overflow-auto::-webkit-scrollbar-thumb:hover {
  @apply bg-gray-400 dark:bg-gray-500;
}
</style>