<template>
  <div class="w-full h-full relative flex bg-white dark:bg-gray-900">
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
          height: `${virtualizer.getTotalSize()}px`,
          position: 'relative',
        }"
      >
        <div
          v-for="virtualItem in virtualizer.getVirtualItems()"
          :key="`line-${virtualItem.key}`"
          class="absolute top-0 left-0 w-full text-right pr-2 text-[13px] font-mono leading-6 select-none cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700"
          :class="{
            'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200 font-semibold': isCurrentSearchLine(virtualItem.index),
            'text-gray-500 dark:text-gray-400': !isCurrentSearchLine(virtualItem.index)
          }"
          :style="{
            height: `${virtualItem.size}px`,
            transform: `translateY(${virtualItem.start}px)`,
            pointerEvents: 'auto',
          }"
          @click="handleLineClick(virtualItem.index)"
          title="点击查看完整行内容"
        >
          <div
            v-if="isCurrentSearchLine(virtualItem.index)"
            class="absolute left-1 top-1/2 transform -translate-y-1/2 w-2 h-2 bg-blue-500 rounded-full"
          />
          {{ startLineNumber + virtualItem.index }}
        </div>
      </div>
    </div>

    <!-- 内容滚动区域 -->
    <div
      ref="containerRef"
      class="flex-1 bg-white dark:bg-gray-900 overflow-auto"
      :class="className"
    >
      <div
        :style="{
          height: `${virtualizer.getTotalSize()}px`,
          position: 'relative',
        }"
      >
        <div
          v-for="virtualItem in virtualizer.getVirtualItems()"
          :key="`content-${virtualItem.key}`"
          class="absolute top-0 left-0 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
          :class="{
            'bg-blue-50 dark:bg-blue-900/10': isCurrentSearchLine(virtualItem.index)
          }"
          :style="{
            height: `${virtualItem.size}px`,
            transform: `translateY(${virtualItem.start}px)`,
          }"
          @click="handleContentClick(virtualItem.index, $event)"
          title="点击查看完整行内容"
        >
          <div
            class="text-[13px] font-mono leading-6 h-full pl-2 pr-4 whitespace-pre text-gray-900 dark:text-gray-100"
          >
            <div class="min-w-max">
              <span
                v-html="renderLineWithHighlight(getLine(virtualItem.index), virtualItem.index)"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 行内容弹窗 -->
    <div
      v-if="modalState.isOpen"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
      @click="closeModal"
    >
      <div
        class="bg-white dark:bg-gray-800 rounded-lg max-w-4xl w-full max-h-[80vh] flex flex-col"
        @click.stop
      >
        <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
            {{ modalState.title }}
          </h3>
          <button
            @click="closeModal"
            class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          <div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
            <pre class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap">{{ modalState.content }}</pre>
          </div>
        </div>
        <div class="p-4 border-t border-gray-200 dark:border-gray-700">
          <p class="text-sm text-gray-500 dark:text-gray-400">
            {{ modalState.description }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'

interface Props {
  content: string
  searchTerm?: string
  onSearchResults?: (results: Array<{ line: number; column: number; text: string; match: string }>, isLimited?: boolean) => void
  onScrollToBottom?: () => void
  onScrollToTop?: (scrollDirection?: 'up' | 'down') => Promise<number | void>
  className?: string
  height?: number
  startLineNumber?: number
  currentSearchIndex?: number
  searchResults?: Array<{ line: number; column: number; text: string; match: string }>
  fileName?: string
}

const props = withDefaults(defineProps<Props>(), {
  searchTerm: '',
  className: '',
  startLineNumber: 1,
  currentSearchIndex: -1,
  searchResults: () => [],
  fileName: ''
})

// 常量配置
const MAX_SEARCH_RESULTS = 1000
const MAX_LINE_LENGTH = 10000
const TRUNCATE_LENGTH = 200
const LONG_LINE_THRESHOLD = 300
const SCROLL_TOP_THRESHOLD = 50
const SCROLL_BOTTOM_THRESHOLD = 100
const SCROLL_DIRECTION_THRESHOLD = 5
const CONSECUTIVE_SCROLL_REQUIRED = 2
const LOAD_LOCK_TIMEOUT = 2000

// 响应式状态
const containerRef = ref<HTMLDivElement>()
const lineNumberRef = ref<HTMLDivElement>()
const modalState = ref<{
  isOpen: boolean
  content?: string
  title?: string
  description?: string
}>({ isOpen: false })

const expandedLongLines = ref<Set<number>>(new Set())
const shouldAdjustScrollAfterPrepend = ref(false)
const scrollAdjustmentData = ref<{
  previousScrollTop: number
  previousLinesCount: number
  visibleStartIndex: number
  scrollOffsetInFirstItem: number
} | null>(null)

const lastScrollTopLoadCheck = ref(-1)
const scrollTopLoadInProgress = ref(false)
const lastScrollTop = ref(0)
const scrollDirection = ref<'up' | 'down' | 'none'>('none')
const consecutiveUpScrollCount = ref(0)

// 计算属性
const lines = computed(() => props.content.split('\n'))

const lineNumberWidth = computed(() => {
  return Math.max(
    40,
    (props.startLineNumber + lines.value.length - 1).toString().length * 8 + 24
  )
})

const searchResultsMap = computed(() => {
  const map = new Map()
  props.searchResults.forEach(result => {
    map.set(result.line, true)
  })
  return map
})

const searchRegex = computed(() => {
  if (!props.searchTerm || props.searchTerm.length < 2) return null
  return new RegExp(`(${props.searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi')
})

// 虚拟化器
const virtualizer = useVirtualizer({
  count: lines.value.length,
  getScrollElement: () => containerRef.value,
  estimateSize: () => 24, // 固定行高
  overscan: 30,
})

// 工具函数
const getLine = (index: number): string => {
  return lines.value[index] || ''
}

const isCurrentSearchLine = (index: number): boolean => {
  const currentLineNumber = props.startLineNumber + index
  return props.currentSearchIndex >= 0 &&
    props.searchResults[props.currentSearchIndex] &&
    props.searchResults[props.currentSearchIndex].line === currentLineNumber
}

const renderLineWithHighlight = (line: string, lineIndex: number): string => {
  const currentLineNumber = props.startLineNumber + lineIndex
  const isLongLine = line.length > LONG_LINE_THRESHOLD
  const isExpanded = expandedLongLines.value.has(lineIndex)

  // 对于超长行，如果未展开则截断显示
  let displayLine = line
  if (isLongLine && !isExpanded && line.length > TRUNCATE_LENGTH) {
    displayLine = line.substring(0, TRUNCATE_LENGTH) + '...'
  }

  // 如果没有搜索词，直接返回
  if (!searchRegex.value) {
    return displayLine
  }

  // 快速查找，避免线性搜索
  if (!searchResultsMap.value.has(currentLineNumber)) {
    return displayLine
  }

  // 获取当前活跃搜索结果的详细信息
  const currentActiveResult = props.currentSearchIndex >= 0 ? props.searchResults[props.currentSearchIndex] : null
  const searchDisplayLine = isLongLine && !isExpanded ? displayLine : line

  // 搜索高亮渲染
  const parts: string[] = []
  let lastIndex = 0
  let match

  searchRegex.value.lastIndex = 0
  while ((match = searchRegex.value.exec(searchDisplayLine)) !== null) {
    // 添加匹配前的文本
    if (match.index > lastIndex) {
      parts.push(searchDisplayLine.slice(lastIndex, match.index))
    }

    // 检查这个匹配是否是当前活跃的匹配
    const isActiveMatch = currentActiveResult &&
      currentActiveResult.line === currentLineNumber &&
      currentActiveResult.column === match.index + 1

    parts.push(
      `<mark class="${isActiveMatch ? 'search-highlight-active' : 'search-highlight'}">${match[0]}</mark>`
    )

    lastIndex = match.index + match[0].length

    // 防止无限循环
    if (match.index === searchRegex.value.lastIndex) {
      searchRegex.value.lastIndex++
    }
  }

  // 添加最后剩余的文本
  if (lastIndex < searchDisplayLine.length) {
    parts.push(searchDisplayLine.slice(lastIndex))
  }

  return parts.join('')
}

// 事件处理
const handleLineClick = (lineIndex: number) => {
  const content = getLine(lineIndex)
  const lineNumber = props.startLineNumber + lineIndex

  modalState.value = {
    isOpen: true,
    content,
    title: `第 ${lineNumber} 行内容`,
    description: `字符数: ${content.length}`,
  }
}

const handleContentClick = (lineIndex: number, event: Event) => {
  const selection = window.getSelection()
  if (selection?.toString().length || (event.target as HTMLElement).closest('button')) {
    return
  }
  handleLineClick(lineIndex)
}

const closeModal = () => {
  modalState.value = { isOpen: false }
}

// 搜索功能
const performSearch = (term: string) => {
  if (!term || term.length < 2) {
    props.onSearchResults?.([], false)
    return
  }

  const results: Array<{ line: number; column: number; text: string; match: string }> = []
  const regex = new RegExp(term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')

  for (let i = 0; i < lines.value.length; i++) {
    if (results.length >= MAX_SEARCH_RESULTS) break

    const line = lines.value[i]
    const searchLine = line.length > MAX_LINE_LENGTH ? line.substring(0, MAX_LINE_LENGTH) : line
    let match

    regex.lastIndex = 0
    while ((match = regex.exec(searchLine)) !== null && results.length < MAX_SEARCH_RESULTS) {
      results.push({
        line: props.startLineNumber + i,
        column: match.index + 1,
        text: line.length > 200 ? line.substring(0, 200) + '...' : line,
        match: match[0],
      })

      if (regex.lastIndex === match.index) regex.lastIndex++
    }
  }

  props.onSearchResults?.(results, results.length >= MAX_SEARCH_RESULTS)
}

// 滚动处理
const handleScroll = async () => {
  const container = containerRef.value
  const lineNumberContainer = lineNumberRef.value
  if (!container) return

  // 同步行号区域滚动
  if (lineNumberContainer) {
    lineNumberContainer.scrollTop = container.scrollTop
  }

  const { scrollTop, scrollHeight, clientHeight } = container

  // 检测滚动方向
  const currentScrollTop = scrollTop
  const scrollDelta = currentScrollTop - lastScrollTop.value

  if (Math.abs(scrollDelta) > SCROLL_DIRECTION_THRESHOLD) {
    if (scrollDelta > 0) {
      scrollDirection.value = 'down'
      consecutiveUpScrollCount.value = 0
    } else {
      scrollDirection.value = 'up'
      consecutiveUpScrollCount.value += 1
    }
  }

  lastScrollTop.value = currentScrollTop

  // 滚动到顶部检测（向前加载）
  if (
    props.onScrollToTop &&
    scrollTop <= SCROLL_TOP_THRESHOLD &&
    !scrollTopLoadInProgress.value &&
    scrollDirection.value === 'up' &&
    consecutiveUpScrollCount.value >= CONSECUTIVE_SCROLL_REQUIRED
  ) {
    // 防抖：避免在相同位置重复触发
    if (Math.abs(scrollTop - lastScrollTopLoadCheck.value) < 10) {
      return
    }

    scrollTopLoadInProgress.value = true
    lastScrollTopLoadCheck.value = scrollTop

    // 记录当前滚动位置
    const currentScrollTop = scrollTop
    const currentLinesCount = lines.value.length
    const virtualItems = virtualizer.getVirtualItems()
    const firstVisibleItem = virtualItems[0]

    try {
      const addedBytes = await props.onScrollToTop(scrollDirection.value)
      if (addedBytes && addedBytes > 0) {
        // 设置滚动调整数据
        scrollAdjustmentData.value = {
          previousScrollTop: currentScrollTop,
          previousLinesCount: currentLinesCount,
          visibleStartIndex: firstVisibleItem?.index || 0,
          scrollOffsetInFirstItem: firstVisibleItem
            ? currentScrollTop - firstVisibleItem.start
            : 0,
        }
        shouldAdjustScrollAfterPrepend.value = true
        consecutiveUpScrollCount.value = 0
      }
    } catch (error) {
      console.error('Error in forward loading:', error)
    } finally {
      setTimeout(() => {
        scrollTopLoadInProgress.value = false
      }, LOAD_LOCK_TIMEOUT)
    }
  }

  // 滚动到底部检测（向后加载）
  if (props.onScrollToBottom && scrollDirection.value === 'down') {
    const isNearBottom = scrollTop + clientHeight >= scrollHeight - SCROLL_BOTTOM_THRESHOLD

    if (isNearBottom) {
      props.onScrollToBottom()
    }
  }
}

// 监听器
watch(() => props.searchTerm, (newTerm) => {
  performSearch(newTerm)
})

watch(() => lines.value.length, (newLength) => {
  // 自动加载
  if (newLength < 30 && props.onScrollToBottom) {
    setTimeout(props.onScrollToBottom, 100)
  }
})

// 内容变化后调整滚动位置
watch([shouldAdjustScrollAfterPrepend, scrollAdjustmentData, () => lines.value.length],
  async ([shouldAdjust, adjustData, linesLength]) => {
    if (shouldAdjust && adjustData) {
      const container = containerRef.value
      if (container) {
        const currentLinesCount = linesLength
        const actualAddedLines = currentLinesCount - adjustData.previousLinesCount

        if (actualAddedLines > 0) {
          const newVisibleStartIndex = adjustData.visibleStartIndex + actualAddedLines

          await nextTick()
          virtualizer.scrollToIndex(newVisibleStartIndex, { align: 'start' })

          setTimeout(() => {
            if (adjustData.scrollOffsetInFirstItem > 0) {
              container.scrollTop += adjustData.scrollOffsetInFirstItem
            }
            lastScrollTop.value = container.scrollTop
            scrollDirection.value = 'none'
            consecutiveUpScrollCount.value = 0
          }, 0)
        }
      }

      shouldAdjustScrollAfterPrepend.value = false
      scrollAdjustmentData.value = null
    }
  }
)

// 生命周期
onMounted(() => {
  const container = containerRef.value
  if (container) {
    container.addEventListener('scroll', handleScroll, { passive: true })
  }
})

onUnmounted(() => {
  const container = containerRef.value
  if (container) {
    container.removeEventListener('scroll', handleScroll)
  }
})

// 暴露给父组件的方法
defineExpose({
  scrollToLine: (lineNumber: number, column?: number) => {
    const targetIndex = lineNumber - props.startLineNumber
    if (targetIndex >= 0 && targetIndex < lines.value.length) {
      virtualizer.scrollToIndex(targetIndex, { align: 'center' })

      if (column && column > 0) {
        setTimeout(() => {
          const container = containerRef.value
          if (container) {
            const charWidth = 7.8
            const targetScrollLeft = Math.max(
              0,
              (column - 1) * charWidth - container.clientWidth / 3
            )
            container.scrollTo({
              left: targetScrollLeft,
              behavior: 'smooth',
            })
          }
        }, 100)
      }
    }
  },
  scrollToPercentage: (percentage: number) => {
    const targetIndex = Math.floor((lines.value.length - 1) * (percentage / 100))
    virtualizer.scrollToIndex(targetIndex, { align: 'start' })
  }
})
</script>

<style scoped>
:deep(.search-highlight) {
  @apply bg-yellow-200 dark:bg-yellow-700;
}

:deep(.search-highlight-active) {
  @apply bg-orange-300 dark:bg-orange-600;
}
</style>