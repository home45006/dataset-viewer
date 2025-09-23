<template>
  <div class="w-full h-full relative flex flex-col bg-white dark:bg-gray-900">
    <!-- 搜索栏 -->
    <div class="flex-shrink-0 p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
      <div class="flex items-center space-x-2">
        <div class="relative flex-1">
          <input
            v-model="searchTerm"
            type="text"
            placeholder="搜索JSON内容..."
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
        <!-- 搜索结果信息和导航 -->
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
    </div>

    <!-- 内容区域 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 固定行号区域 -->
      <div
        ref="lineNumberRef"
        class="flex-shrink-0 bg-gray-50 dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 overflow-hidden relative z-10"
        :style="{
          width: `${lineNumberWidth}px`,
          pointerEvents: 'none',
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
          class="absolute top-0 left-0 w-full text-right pr-2 text-[13px] font-mono leading-6 select-none"
          :class="{
            'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200 font-semibold': isSearchResultLine(virtualItem.index),
            'text-gray-500 dark:text-gray-400': !isSearchResultLine(virtualItem.index)
          }"
          :style="{
            height: `${virtualItem.size}px`,
            transform: `translateY(${virtualItem.start}px)`,
          }"
        >
          {{ virtualItem.index + 1 }}
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
          class="absolute top-0 left-0 w-full"
          :class="{
            'bg-yellow-50 dark:bg-yellow-900/10': isSearchResultLine(virtualItem.index)
          }"
          :style="{
            height: `${virtualItem.size}px`,
            transform: `translateY(${virtualItem.start}px)`,
          }"
        >
          <div
            class="text-[13px] font-mono leading-6 h-full pl-2 pr-4 whitespace-pre text-gray-900 dark:text-gray-100"
            v-html="getHighlightedLineWithSearch(virtualItem.index)"
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

// 将内容按行分割
const lines = computed(() => {
  return props.content.split('\n')
})

// 计算行号宽度
const updateLineNumberWidth = () => {
  const totalLines = lines.value.length
  const digits = totalLines.toString().length
  lineNumberWidth.value = Math.max(40, digits * 8 + 16)
}

// 高亮状态
const highlightedLines = ref<string[]>([])

// 初始化语法高亮
const initializeHighlighting = async () => {
  try {
    const highlighted = await highlightCode(props.content, props.fileName)
    // 从HTML中提取每行的高亮代码
    const tempDiv = document.createElement('div')
    tempDiv.innerHTML = highlighted

    // 移除外层的pre标签，只保留内容
    const preElement = tempDiv.querySelector('pre')
    if (preElement) {
      highlightedLines.value = preElement.innerHTML.split('\n')
    } else {
      highlightedLines.value = lines.value
    }
  } catch (error) {
    console.error('语法高亮失败:', error)
    highlightedLines.value = lines.value
  }
}

// 获取高亮的行内容
const getHighlightedLine = (index: number): string => {
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
const getHighlightedLineWithSearch = (index: number): string => {
  let lineContent = getHighlightedLine(index)

  if (!searchTerm.value.trim() || !isSearchResultLine(index)) {
    return lineContent
  }

  // 找到该行的所有搜索结果
  const lineResults = searchResults.value.filter(result => result.lineIndex === index)

  // 从后往前替换，避免位置偏移问题
  lineResults.reverse().forEach(result => {
    const isCurrentResult = searchResults.value[currentSearchIndex.value]?.lineIndex === index &&
                           searchResults.value[currentSearchIndex.value]?.startPos === result.startPos

    const originalText = lines.value[index]
    const beforeText = originalText.substring(0, result.startPos)
    const matchText = originalText.substring(result.startPos, result.endPos)
    const afterText = originalText.substring(result.endPos)

    // 高亮当前搜索结果
    const highlightClass = isCurrentResult
      ? 'bg-yellow-300 dark:bg-yellow-600 text-black dark:text-white font-bold'
      : 'bg-yellow-200 dark:bg-yellow-800 text-gray-900 dark:text-gray-100'

    const highlightedMatch = `<span class="${highlightClass}">${escapeHtml(matchText)}</span>`

    // 在高亮的HTML中进行替换需要更复杂的处理
    // 这里简化处理，直接在原始文本基础上加高亮
    if (lineContent === originalText) {
      lineContent = escapeHtml(beforeText) + highlightedMatch + escapeHtml(afterText)
    }
  })

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
    count: lines.value.length,
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

// 监听搜索词变化
watch(searchTerm, () => {
  performSearch()
}, { immediate: false })

onMounted(async () => {
  updateLineNumberWidth()
  await nextTick()
  await initializeHighlighting()
})
</script>

<style scoped>
/* 确保语法高亮的span元素显示正确 */
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