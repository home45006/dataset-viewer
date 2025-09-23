<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
    @click="$emit('close')"
  >
    <div
      class="bg-white dark:bg-gray-800 rounded-lg max-w-4xl w-full max-h-[80vh] flex flex-col shadow-2xl"
      @click.stop
    >
      <!-- 头部 -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center space-x-3">
          <div class="w-8 h-8 bg-blue-100 dark:bg-blue-900 rounded-lg flex items-center justify-center">
            <svg class="w-4 h-4 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-1l-4 4z"/>
            </svg>
          </div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
              {{ title }}
            </h3>
            <p v-if="description" class="text-sm text-gray-500 dark:text-gray-400">
              {{ description }}
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
      <div class="flex-1 overflow-hidden">
        <div class="h-full overflow-auto p-4">
          <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
            <!-- 语法高亮的内容 -->
            <div
              v-if="highlightedContent"
              class="text-sm font-mono leading-relaxed"
              v-html="highlightedContent"
            />
            <!-- 普通文本内容 -->
            <pre
              v-else
              class="text-sm font-mono leading-relaxed text-gray-800 dark:text-gray-200 whitespace-pre-wrap"
            >{{ content }}</pre>

            <!-- 搜索高亮 -->
            <div
              v-if="searchTerm && searchMatches.length > 0"
              class="mt-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-md border-l-4 border-yellow-400"
            >
              <h4 class="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-2">
                搜索匹配 ({{ searchMatches.length }} 个)
              </h4>
              <div class="space-y-1">
                <div
                  v-for="(match, index) in searchMatches"
                  :key="index"
                  class="text-xs text-yellow-700 dark:text-yellow-300"
                >
                  位置 {{ match.start + 1 }}: "<mark class="bg-yellow-200 dark:bg-yellow-700 px-1 rounded">{{ match.text }}</mark>"
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4 text-sm text-gray-500 dark:text-gray-400">
            <span>字符数: {{ content.length.toLocaleString() }}</span>
            <span v-if="searchTerm">搜索词: "{{ searchTerm }}"</span>
            <span v-if="fileName">文件: {{ fileName }}</span>
          </div>
          <div class="flex space-x-2">
            <button
              @click="copyToClipboard"
              class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
            >
              复制内容
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
import { ref, computed, watch } from 'vue'
import { highlightLine, getLanguageFromFileName, isLanguageSupported } from '../../utils/syntaxHighlighter'

interface Props {
  isOpen: boolean
  content: string
  title: string
  description?: string
  searchTerm?: string
  fileName?: string
}

const props = withDefaults(defineProps<Props>(), {
  description: '',
  searchTerm: '',
  fileName: ''
})

defineEmits<{
  close: []
}>()

const highlightedContent = ref('')

// 搜索匹配
const searchMatches = computed(() => {
  if (!props.searchTerm || props.searchTerm.length < 2) return []

  const matches = []
  const regex = new RegExp(props.searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')
  let match

  while ((match = regex.exec(props.content)) !== null) {
    matches.push({
      start: match.index,
      end: match.index + match[0].length,
      text: match[0]
    })
    if (match.index === regex.lastIndex) regex.lastIndex++
  }

  return matches
})

// 语法高亮
const performHighlighting = async () => {
  if (!props.fileName || !props.content.trim()) {
    highlightedContent.value = ''
    return
  }

  const language = getLanguageFromFileName(props.fileName)
  if (!isLanguageSupported(language)) {
    highlightedContent.value = ''
    return
  }

  try {
    const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light'
    highlightedContent.value = await highlightLine(props.content, language, theme)
  } catch (error) {
    console.warn('Failed to highlight content:', error)
    highlightedContent.value = ''
  }
}

// 监听内容和文件名变化
watch([() => props.content, () => props.fileName, () => props.isOpen], () => {
  if (props.isOpen) {
    performHighlighting()
  }
}, { immediate: true })

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

/* 语法高亮内容样式 */
:deep(.shiki) {
  background: transparent !important;
}

:deep(.shiki code) {
  background: transparent !important;
}
</style>