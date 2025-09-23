<template>
  <div class="w-full h-full relative flex flex-col bg-white dark:bg-gray-900">
    <!-- 工具栏 -->
    <div class="flex-shrink-0 p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
      <div class="flex items-center justify-between">
        <!-- 搜索区域 -->
        <div class="flex items-center space-x-4 flex-1">
          <div class="relative flex-1 max-w-md">
            <input
              v-model="searchTerm"
              type="text"
              placeholder="搜索日志内容..."
              class="w-full px-3 py-2 pr-10 text-sm border border-gray-300 dark:border-gray-600 rounded-lg
                     bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100
                     focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              @keydown.enter="searchNext"
              @keydown.shift.enter="searchPrevious"
            />
            <div class="absolute inset-y-0 right-0 flex items-center pr-3">
              <svg v-if="!searchTerm" class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
              </svg>
              <button
                v-else
                @click="clearSearch"
                class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
              </button>
            </div>
          </div>
          <!-- 搜索结果导航 -->
          <div v-if="searchResults.length > 0" class="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
            <span>{{ currentSearchIndex + 1 }}/{{ searchResults.length }}</span>
            <div class="flex space-x-1">
              <button
                @click="searchPrevious"
                :disabled="searchResults.length === 0"
                class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                title="上一个结果 (Shift+Enter)"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/>
                </svg>
              </button>
              <button
                @click="searchNext"
                :disabled="searchResults.length === 0"
                class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                title="下一个结果 (Enter)"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
              </button>
            </div>
          </div>
          <div v-else-if="searchTerm && searchResults.length === 0" class="text-sm text-gray-500 dark:text-gray-400">
            无结果
          </div>
        </div>

        <!-- 日志级别过滤器 -->
        <div class="flex items-center space-x-2">
          <span class="text-sm text-gray-600 dark:text-gray-400">级别:</span>
          <select
            v-model="selectedLogLevel"
            class="text-xs border border-gray-300 dark:border-gray-600 rounded px-2 py-1
                   bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
          >
            <option value="all">全部</option>
            <option value="error">ERROR</option>
            <option value="warn">WARN</option>
            <option value="info">INFO</option>
            <option value="debug">DEBUG</option>
            <option value="trace">TRACE</option>
          </select>
        </div>

        <!-- 时间范围过滤器 -->
        <div class="flex items-center space-x-2">
          <span class="text-sm text-gray-600 dark:text-gray-400">时间:</span>
          <select
            v-model="selectedTimeRange"
            class="text-xs border border-gray-300 dark:border-gray-600 rounded px-2 py-1
                   bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
          >
            <option value="all">全部</option>
            <option value="today">今天</option>
            <option value="hour">最近1小时</option>
            <option value="30min">最近30分钟</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 日志内容区域 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 固定行号区域 -->
      <div
        ref="lineNumberRef"
        class="flex-shrink-0 bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-hidden relative z-10"
        :style="{
          width: `${lineNumberWidth}px`,
        }"
      >
        <div
          :style="{
            height: `${virtualizer?.getTotalSize() || 0}px`,
            position: 'relative',
          }"
        >
          <div
            v-for="virtualItem in virtualizer?.getVirtualItems() || []"
            :key="`line-${virtualItem.key}`"
            class="absolute top-0 left-0 w-full text-right pr-2 text-[13px] font-mono leading-6 select-none flex items-center justify-end"
            :class="{
              'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 font-semibold': isSearchResultLine(filteredLines[virtualItem.index]?.originalIndex),
              'text-gray-500 dark:text-gray-400': !isSearchResultLine(filteredLines[virtualItem.index]?.originalIndex)
            }"
            :style="{
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }"
          >
            <span class="select-none">{{ (filteredLines[virtualItem.index]?.originalIndex ?? -1) + 1 }}</span>
          </div>
        </div>
      </div>

      <!-- 内容滚动区域 -->
      <div
        ref="containerRef"
        class="flex-1 bg-white dark:bg-gray-900 overflow-auto"
      >
        <div
          :style="{
            height: `${virtualizer?.getTotalSize() || 0}px`,
            position: 'relative',
          }"
        >
          <div
            v-for="virtualItem in virtualizer?.getVirtualItems() || []"
            :key="`content-${virtualItem.key}`"
            class="absolute top-0 left-0 w-full log-line"
            :class="{
              'bg-yellow-50 dark:bg-yellow-900/10': isSearchResultLine(filteredLines[virtualItem.index]?.originalIndex),
              'bg-red-50 dark:bg-red-900/20': getLogLevel(filteredLines[virtualItem.index]?.content) === 'error',
              'bg-orange-50 dark:bg-orange-900/20': getLogLevel(filteredLines[virtualItem.index]?.content) === 'warn',
              'bg-blue-50 dark:bg-blue-900/20': getLogLevel(filteredLines[virtualItem.index]?.content) === 'info',
            }"
            :style="{
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`,
            }"
          >
            <div
              class="text-[13px] font-mono leading-6 h-full pl-2 pr-4 whitespace-pre text-gray-900 dark:text-gray-100"
              v-html="getHighlightedLineWithSearch(filteredLines[virtualItem.index]?.originalIndex, filteredLines[virtualItem.index]?.content)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { highlightCode, getFileLanguage } from '@/utils/shikiHighlighter'

interface Props {
  content: string
  fileName?: string
  height?: number
}

const props = withDefaults(defineProps<Props>(), {
  fileName: '',
  height: 400
})

// 行号宽度
const lineNumberWidth = ref(50)
const containerRef = ref<HTMLElement>()
const lineNumberRef = ref<HTMLElement>()

// 搜索相关状态
const searchTerm = ref('')
const searchResults = ref<{ lineIndex: number, startPos: number, endPos: number }[]>([])
const currentSearchIndex = ref(-1)

// 日志过滤状态
const selectedLogLevel = ref('all')
const selectedTimeRange = ref('all')

// 高亮状态
const highlightedLines = ref<string[]>([])

// 检测深色模式
const isDarkMode = computed(() => {
  if (typeof window === 'undefined') return false
  return document.documentElement.classList.contains('dark') ||
         window.matchMedia('(prefers-color-scheme: dark)').matches
})

// 将内容按行分割
const lines = computed(() => {
  return props.content.split('\n')
})

// 日志级别检测
const getLogLevel = (line: string): string => {
  if (!line) return 'unknown'
  const upperLine = line.toUpperCase()
  if (upperLine.includes('ERROR') || upperLine.includes('FATAL') || upperLine.includes('EXCEPTION')) return 'error'
  if (upperLine.includes('WARN') || upperLine.includes('WARNING')) return 'warn'
  if (upperLine.includes('INFO')) return 'info'
  if (upperLine.includes('DEBUG')) return 'debug'
  if (upperLine.includes('TRACE')) return 'trace'
  return 'unknown'
}

// 时间戳检测
const getLogTimestamp = (line: string): Date | null => {
  // 常见的时间戳格式
  const timePatterns = [
    /(\d{4}-\d{2}-\d{2}[\sT]\d{2}:\d{2}:\d{2})/,  // 2024-01-01 12:00:00 或 2024-01-01T12:00:00
    /(\d{2}\/\d{2}\/\d{4}\s+\d{2}:\d{2}:\d{2})/,  // 01/01/2024 12:00:00
    /(\d{2}-\d{2}-\d{4}\s+\d{2}:\d{2}:\d{2})/,    // 01-01-2024 12:00:00
  ]

  for (const pattern of timePatterns) {
    const match = line.match(pattern)
    if (match) {
      const dateStr = match[1].replace('T', ' ')
      const date = new Date(dateStr)
      if (!isNaN(date.getTime())) {
        return date
      }
    }
  }
  return null
}

// 过滤后的行
const filteredLines = computed(() => {
  const now = new Date()
  const result: { originalIndex: number, content: string }[] = []

  lines.value.forEach((line, index) => {
    // 日志级别过滤
    if (selectedLogLevel.value !== 'all') {
      const logLevel = getLogLevel(line)
      if (logLevel !== selectedLogLevel.value) {
        return
      }
    }

    // 时间范围过滤
    if (selectedTimeRange.value !== 'all') {
      const timestamp = getLogTimestamp(line)
      if (timestamp) {
        const timeDiff = now.getTime() - timestamp.getTime()
        switch (selectedTimeRange.value) {
          case 'today':
            if (timeDiff > 24 * 60 * 60 * 1000) return
            break
          case 'hour':
            if (timeDiff > 60 * 60 * 1000) return
            break
          case '30min':
            if (timeDiff > 30 * 60 * 1000) return
            break
        }
      }
    }

    result.push({ originalIndex: index, content: line })
  })

  return result
})

// 计算行号宽度
const updateLineNumberWidth = () => {
  const totalLines = lines.value.length
  const digits = totalLines.toString().length
  lineNumberWidth.value = Math.max(40, digits * 8 + 16)
}

// 初始化语法高亮
const initializeHighlighting = async () => {
  try {
    const highlighted = await highlightCode(props.content, props.fileName, isDarkMode.value)

    if (highlighted) {
      const tempDiv = document.createElement('div')
      tempDiv.innerHTML = highlighted

      const preElement = tempDiv.querySelector('pre.shiki, pre')
      if (preElement) {
        const codeElement = preElement.querySelector('code')
        if (codeElement) {
          highlightedLines.value = codeElement.innerHTML.split('\n')
        } else {
          highlightedLines.value = preElement.innerHTML.split('\n')
        }
      } else {
        highlightedLines.value = tempDiv.innerHTML.split('\n')
      }
    } else {
      highlightedLines.value = lines.value
    }
  } catch (error) {
    console.error('日志语法高亮失败:', error)
    highlightedLines.value = lines.value
  }

  console.log(`日志语法高亮初始化完成: ${highlightedLines.value.length} 行, 主题: ${isDarkMode.value ? 'dark' : 'light'}`)
}

// 获取高亮的行内容
const getHighlightedLine = (index: number): string => {
  if (index === undefined || index < 0) return ''
  if (highlightedLines.value.length > 0) {
    return highlightedLines.value[index] || ''
  }
  return lines.value[index] || ''
}

// 搜索功能
const performSearch = () => {
  searchResults.value = []
  currentSearchIndex.value = -1

  if (!searchTerm.value.trim()) {
    return
  }

  const searchRegex = new RegExp(escapeRegExp(searchTerm.value), 'gi')

  lines.value.forEach((line, lineIndex) => {
    let match
    while ((match = searchRegex.exec(line)) !== null) {
      searchResults.value.push({
        lineIndex,
        startPos: match.index,
        endPos: match.index + match[0].length
      })
    }
  })

  if (searchResults.value.length > 0) {
    currentSearchIndex.value = 0
    scrollToSearchResult(0)
  }
}

// 转义正则表达式特殊字符
const escapeRegExp = (string: string): string => {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

// 清除搜索
const clearSearch = () => {
  searchTerm.value = ''
  searchResults.value = []
  currentSearchIndex.value = -1
}

// 下一个搜索结果
const searchNext = () => {
  if (searchResults.value.length === 0) return

  currentSearchIndex.value = (currentSearchIndex.value + 1) % searchResults.value.length
  scrollToSearchResult(currentSearchIndex.value)
}

// 上一个搜索结果
const searchPrevious = () => {
  if (searchResults.value.length === 0) return

  currentSearchIndex.value = currentSearchIndex.value <= 0
    ? searchResults.value.length - 1
    : currentSearchIndex.value - 1
  scrollToSearchResult(currentSearchIndex.value)
}

// 滚动到搜索结果
const scrollToSearchResult = (index: number) => {
  if (!searchResults.value[index] || !virtualizer.value) return

  const result = searchResults.value[index]
  const itemSize = 24 // 每行高度
  const targetOffset = result.lineIndex * itemSize

  virtualizer.value.scrollToOffset(targetOffset)
}

// 判断是否为搜索结果行
const isSearchResultLine = (lineIndex: number): boolean => {
  return searchResults.value.some(result => result.lineIndex === lineIndex)
}

// 获取带搜索高亮的行内容
const getHighlightedLineWithSearch = (index: number, content: string): string => {
  if (index === undefined || index < 0 || !content) return ''

  let lineContent = getHighlightedLine(index)

  // 如果没有搜索词或该行没有搜索结果，直接返回语法高亮的内容
  if (!searchTerm.value.trim() || !isSearchResultLine(index)) {
    return lineContent || content
  }

  // 如果语法高亮内容为空，回退到原始文本
  if (!lineContent) {
    lineContent = content
  }

  try {
    // 添加搜索高亮
    const searchRegex = new RegExp(escapeRegExp(searchTerm.value), 'gi')
    lineContent = lineContent.replace(searchRegex, (match) => {
      const isCurrentResult = searchResults.value[currentSearchIndex.value]?.lineIndex === index
      const highlightClass = isCurrentResult
        ? 'bg-yellow-300 dark:bg-yellow-600 text-black dark:text-white font-bold search-highlight-current'
        : 'bg-yellow-200 dark:bg-yellow-800 search-highlight'
      return `<span class="${highlightClass}">${match}</span>`
    })
  } catch (error) {
    console.warn('搜索高亮处理失败:', error)
  }

  return lineContent
}

// HTML转义
const escapeHtml = (text: string): string => {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

// 虚拟滚动设置
const virtualizer = useVirtualizer(
  computed(() => ({
    count: filteredLines.value.length,
    getScrollElement: () => containerRef.value,
    estimateSize: () => 24, // 每行高度
    overscan: 10,
  }))
)

// 同步滚动
watch(() => virtualizer.value?.scrollOffset, (newOffset) => {
  if (lineNumberRef.value && newOffset !== undefined) {
    lineNumberRef.value.scrollTop = newOffset
  }
})

// 监听内容变化
watch(() => props.content, async () => {
  updateLineNumberWidth()
  await nextTick()
  await initializeHighlighting()
}, { immediate: true })

// 监听深色模式变化
watch(isDarkMode, async () => {
  await initializeHighlighting()
})

// 监听搜索词变化
watch(searchTerm, () => {
  performSearch()
}, { immediate: false })

// 监听过滤器变化
watch([selectedLogLevel, selectedTimeRange], () => {
  // 重新计算过滤结果时，清除搜索状态
  searchResults.value = []
  currentSearchIndex.value = -1
  if (searchTerm.value.trim()) {
    performSearch()
  }
})

onMounted(async () => {
  updateLineNumberWidth()
  await nextTick()
  await initializeHighlighting()
})
</script>

<style scoped>
/* 日志行显示优化 */
.log-line {
  overflow: hidden;
  text-overflow: ellipsis;
  border-left: 3px solid transparent;
}

/* 日志级别颜色指示器 */
.log-line:has([class*="ERROR"]) {
  border-left-color: #ef4444;
}

.log-line:has([class*="WARN"]) {
  border-left-color: #f59e0b;
}

.log-line:has([class*="INFO"]) {
  border-left-color: #3b82f6;
}

.log-line:has([class*="DEBUG"]) {
  border-left-color: #6b7280;
}

/* 确保语法高亮元素在虚拟化容器中正确显示 */
.log-line :deep(span) {
  font-family: inherit !important;
  font-size: inherit !important;
  line-height: inherit !important;
}

/* 搜索高亮样式 */
:deep(.search-highlight) {
  border-radius: 2px;
  padding: 0 1px;
  transition: all 0.2s ease;
}

:deep(.search-highlight-current) {
  border-radius: 2px;
  padding: 0 1px;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.2);
  animation: pulse 1s ease-in-out;
}

@keyframes pulse {
  0% { opacity: 0.8; }
  50% { opacity: 1; }
  100% { opacity: 0.8; }
}

/* 确保语法高亮的span元素显示正确 */
:deep(.shiki) {
  font-family: 'JetBrains Mono', 'Fira Code', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace !important;
  font-size: 13px;
  line-height: 1.5;
  background: transparent !important;
}

:deep(.shiki span) {
  font-family: inherit !important;
}

/* 滚动条样式 */
:deep(.overflow-auto::-webkit-scrollbar) {
  width: 12px;
  height: 12px;
}

:deep(.overflow-auto::-webkit-scrollbar-track) {
  background: #f1f1f1;
  border-radius: 6px;
}

:deep(.overflow-auto::-webkit-scrollbar-thumb) {
  background: #c1c1c1;
  border-radius: 6px;
}

:deep(.overflow-auto::-webkit-scrollbar-thumb:hover) {
  background: #a8a8a8;
}

.dark :deep(.overflow-auto::-webkit-scrollbar-track) {
  background: #374151;
}

.dark :deep(.overflow-auto::-webkit-scrollbar-thumb) {
  background: #6b7280;
}

.dark :deep(.overflow-auto::-webkit-scrollbar-thumb:hover) {
  background: #9ca3af;
}
</style>