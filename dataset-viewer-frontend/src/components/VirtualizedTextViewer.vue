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
          height: `${virtualizer.value?.getTotalSize() || 0}px`,
          position: 'relative',
        }"
      >
        <div
          v-for="virtualItem in virtualizer.value?.getVirtualItems() || []"
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
          @click="handleLineClick(getVisibleLineOriginalIndex(virtualItem.index))"
          title="点击查看完整行内容"
        >
          <div
            v-if="isCurrentSearchLine(virtualItem.index)"
            class="absolute left-1 top-1/2 transform -translate-y-1/2 w-2 h-2 bg-blue-500 rounded-full"
          />
          {{ startLineNumber + getVisibleLineOriginalIndex(virtualItem.index) }}
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
          height: `${virtualizer.value?.getTotalSize() || 0}px`,
          position: 'relative',
        }"
      >
        <div
          v-for="virtualItem in virtualizer.value?.getVirtualItems() || []"
          :key="`content-${virtualItem.key}`"
          class="absolute top-0 left-0 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
          :class="{
            'bg-blue-50 dark:bg-blue-900/10': isCurrentSearchLine(virtualItem.index)
          }"
          :style="{
            height: `${virtualItem.size}px`,
            transform: `translateY(${virtualItem.start}px)`,
          }"
          @click="handleContentClick(getVisibleLineOriginalIndex(virtualItem.index), $event)"
          title="点击查看完整行内容"
        >
          <div
            class="text-[13px] font-mono leading-6 h-full pl-2 pr-4 whitespace-pre"
            :class="shouldHighlight ? '' : 'text-gray-900 dark:text-gray-100'"
          >
            <div class="min-w-max flex items-center">
              <span
                v-html="renderLineWithHighlight(getVisibleLine(virtualItem.index), getVisibleLineOriginalIndex(virtualItem.index))"
              />
              <!-- 代码折叠指示器 -->
              <FoldingIndicator
                v-if="getFoldableRangeAtLine(getVisibleLineOriginalIndex(virtualItem.index))"
                :is-collapsed="isRangeCollapsed(getVisibleLineOriginalIndex(virtualItem.index))"
                @toggle="toggleFoldingRange(getVisibleLineOriginalIndex(virtualItem.index))"
              />
              <!-- 折叠摘要信息 -->
              <div
                v-if="isRangeCollapsed(getVisibleLineOriginalIndex(virtualItem.index))"
                class="ml-2 text-xs text-gray-500 dark:text-gray-400 italic"
              >
                {{ getFoldableRangeAtLine(getVisibleLineOriginalIndex(virtualItem.index))?.summary }}
              </div>
              <!-- 大节点指示器 -->
              <div
                v-if="isLargeNode(getVisibleLineOriginalIndex(virtualItem.index))"
                class="ml-2 px-1 text-xs bg-orange-100 dark:bg-orange-900 text-orange-600 dark:text-orange-300 rounded"
              >
                大节点 ({{ getLargeNodeLineCount(getVisibleLineOriginalIndex(virtualItem.index)) }} 行)
              </div>
              <!-- 超长行展开按钮 -->
              <button
                v-if="isLongLineWithButton(getVisibleLineOriginalIndex(virtualItem.index))"
                @click.stop="toggleLongLineExpansion(getVisibleLineOriginalIndex(virtualItem.index))"
                class="ml-2 px-1.5 py-0.5 text-xs bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
              >
                {{ expandedLongLines.has(getVisibleLineOriginalIndex(virtualItem.index)) ? '收起长行' : '展开长行' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 行内容弹窗 -->
    <LineContentModal
      :is-open="modalState.isOpen"
      :content="modalState.content || ''"
      :title="modalState.title || ''"
      :description="modalState.description"
      :search-term="searchTerm"
      :file-name="fileName"
      @close="closeModal"
    />

    <!-- Markdown 预览弹窗 -->
    <MarkdownPreview
      :is-open="isMarkdownPreviewOpen"
      :content="content"
      :file-name="fileName"
      @close="setIsMarkdownPreviewOpen(false)"
    />

  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useFoldingLogic } from '../composables/useFoldingLogic'
import { useSyntaxHighlighting } from '../composables/useSyntaxHighlighting'
import { highlightLine, getLanguageFromFileName, isLanguageSupported } from '../utils/syntaxHighlighter'
import FoldingIndicator from './text-viewer/FoldingIndicator.vue'
import LineContentModal from './text-viewer/LineContentModal.vue'
import MarkdownPreview from './text-viewer/MarkdownPreview.vue'

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
  isMarkdown?: boolean
  isMarkdownPreviewOpen?: boolean
  setIsMarkdownPreviewOpen?: (open: boolean) => void
}

const props = withDefaults(defineProps<Props>(), {
  searchTerm: '',
  className: '',
  startLineNumber: 1,
  currentSearchIndex: -1,
  searchResults: () => [],
  fileName: '',
  isMarkdown: false,
  isMarkdownPreviewOpen: false,
  setIsMarkdownPreviewOpen: () => {}
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

// 使用语法高亮 hook
const { enabled: syntaxHighlightingEnabled } = useSyntaxHighlighting()

// 响应式状态
const containerRef = ref<HTMLDivElement>()
const lineNumberRef = ref<HTMLDivElement>()
const modalState = ref<{
  isOpen: boolean
  content?: string
  title?: string
  description?: string
}>({ isOpen: false })

const highlightedLines = ref<Map<number, string>>(new Map())
const isHighlighting = ref(false)
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
const lines = computed(() => {
  console.log('📊 VirtualizedTextViewer received props:', {
    contentLength: props.content?.length || 0,
    fileName: props.fileName,
    hasContent: !!props.content,
    contentPreview: props.content?.substring(0, 100) + '...'
  })
  const result = props.content.split('\n')
  console.log('📊 Lines computed:', result.length, 'lines')
  return result
})

// 可见范围状态
const visibleRange = ref<{ start: number; end: number }>({
  start: 0,
  end: 100,
})

// 临时简化：先不使用代码折叠，直接显示所有行，专注于修复语法高亮
const supportsFolding = ref(false)
const foldableRanges = ref([])
const collapsedRanges = ref(new Set())
const visibleLines = computed(() => {
  const result = lines.value.map((line, index) => ({ line, originalIndex: index }))
  console.log('📊 visibleLines computed:', {
    totalLines: result.length,
    firstLine: result[0]?.line?.substring(0, 50) + '...',
    lastLine: result[result.length - 1]?.line?.substring(0, 50) + '...'
  })
  return result
})
const getFoldableRangeAtLine = () => null
const toggleFoldingRangeById = () => {}

const lineNumberWidth = computed(() => {
  return Math.max(
    40,
    (props.startLineNumber + lines.value.length - 1).toString().length * 8 + 24
  )
})

// 语法高亮相关
const detectedLanguage = computed(() => {
  return syntaxHighlightingEnabled.value && props.fileName
    ? getLanguageFromFileName(props.fileName)
    : 'text'
})

const shouldHighlight = computed(() => {
  return syntaxHighlightingEnabled.value && isLanguageSupported(detectedLanguage.value)
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

// 虚拟化器 - 简化版本
const virtualizerOptions = computed(() => ({
  count: lines.value.length,
  getScrollElement: () => containerRef.value,
  estimateSize: () => 24,
  overscan: 5,
}))

console.log('📊 Creating virtualizer with options:', {
  count: lines.value.length,
  hasContainer: !!containerRef.value
})

const virtualizer = useVirtualizer(virtualizerOptions)

// 添加virtualizer状态监听
watch(() => virtualizer.value, (newVirtualizer) => {
  console.log('📊 Virtualizer watch triggered:', {
    hasVirtualizer: !!newVirtualizer,
    type: typeof newVirtualizer
  })
  if (newVirtualizer) {
    try {
      console.log('📊 Virtualizer initialized successfully:', {
        totalSize: newVirtualizer.getTotalSize(),
        virtualItems: newVirtualizer.getVirtualItems().length
      })
    } catch (error) {
      console.error('📊 Error accessing virtualizer methods:', error)
    }
  }
}, { immediate: true })

// 组件挂载后检查DOM状态
onMounted(() => {
  console.log('📊 VirtualizedTextViewer mounted:', {
    containerExists: !!containerRef.value,
    containerHeight: containerRef.value?.offsetHeight,
    containerWidth: containerRef.value?.offsetWidth,
    virtualizerValue: !!virtualizer.value,
    linesLength: lines.value.length
  })
})

// 工具函数
const getLine = (index: number): string => {
  return lines.value[index] || ''
}

const getVisibleLine = (virtualIndex: number): string => {
  const visibleLine = visibleLines.value[virtualIndex]
  return visibleLine ? visibleLine.line : ''
}

const getVisibleLineOriginalIndex = (virtualIndex: number): number => {
  const visibleLine = visibleLines.value[virtualIndex]
  return visibleLine ? visibleLine.originalIndex : virtualIndex
}

const isCurrentSearchLine = (virtualIndex: number): boolean => {
  const originalIndex = getVisibleLineOriginalIndex(virtualIndex)
  const currentLineNumber = props.startLineNumber + originalIndex
  return props.currentSearchIndex >= 0 &&
    props.searchResults[props.currentSearchIndex] &&
    props.searchResults[props.currentSearchIndex].line === currentLineNumber
}

// 代码折叠相关函数
const isRangeCollapsed = (lineIndex: number): boolean => {
  const range = getFoldableRangeAtLine(lineIndex)
  return range ? collapsedRanges.value.has(range.id) : false
}

const toggleFoldingRange = (lineIndex: number): void => {
  const range = getFoldableRangeAtLine(lineIndex)
  if (range) {
    toggleFoldingRangeById(range.id)
  }
}

const isLargeNode = (lineIndex: number): boolean => {
  const range = getFoldableRangeAtLine(lineIndex)
  return range ? !collapsedRanges.value.has(range.id) && (range.endLine - range.startLine > 100) : false
}

const getLargeNodeLineCount = (lineIndex: number): number => {
  const range = getFoldableRangeAtLine(lineIndex)
  return range ? range.endLine - range.startLine + 1 : 0
}

// 超长行处理
const isLongLineWithButton = (lineIndex: number): boolean => {
  const line = lines.value[lineIndex] || ''
  const isLongLine = line.length > LONG_LINE_THRESHOLD
  return isLongLine && line.length > TRUNCATE_LENGTH
}

const toggleLongLineExpansion = (lineIndex: number): void => {
  const newSet = new Set(expandedLongLines.value)
  if (newSet.has(lineIndex)) {
    newSet.delete(lineIndex)
  } else {
    newSet.add(lineIndex)
  }
  expandedLongLines.value = newSet
}

// 高亮可见行的异步处理
const highlightVisibleLines = async (virtualItems: any[]) => {
  if (!shouldHighlight.value || isHighlighting.value) return

  isHighlighting.value = true
  const lineIndexesToHighlight: number[] = []

  // 找出需要高亮但尚未缓存的行
  virtualItems.forEach(item => {
    const originalIndex = getVisibleLineOriginalIndex(item.index)
    const line = lines.value[originalIndex] || ''
    if (!highlightedLines.value.has(originalIndex) && line.length < MAX_LINE_LENGTH) {
      lineIndexesToHighlight.push(originalIndex)
    }
  })

  if (lineIndexesToHighlight.length === 0) {
    isHighlighting.value = false
    return
  }

  try {
    const theme = document.documentElement.classList.contains('dark') ? 'dark' : 'light'
    const linesToHighlight = lineIndexesToHighlight.map(index => lines.value[index] || '')
    const results = await Promise.all(
      linesToHighlight.map(line =>
        highlightLine(line, detectedLanguage.value, theme)
      )
    )

    const newMap = new Map(highlightedLines.value)
    lineIndexesToHighlight.forEach((lineIndex, i) => {
      newMap.set(lineIndex, results[i])
    })
    highlightedLines.value = newMap
  } catch (error) {
    console.error('Error highlighting lines:', error)
  } finally {
    isHighlighting.value = false
  }
}

const renderLineWithHighlight = (line: string, originalLineIndex: number): string => {
  const currentLineNumber = props.startLineNumber + originalLineIndex
  const isLongLine = line.length > LONG_LINE_THRESHOLD
  const isExpanded = expandedLongLines.value.has(originalLineIndex)

  // 对于超长行，如果未展开则截断显示
  let displayLine = line
  if (isLongLine && !isExpanded && line.length > TRUNCATE_LENGTH) {
    displayLine = line.substring(0, TRUNCATE_LENGTH) + '...'
  }

  // 获取语法高亮的内容
  let processedLine = displayLine
  let isHighlighted = false

  if (
    shouldHighlight.value &&
    highlightedLines.value.has(originalLineIndex) &&
    (line.length < MAX_LINE_LENGTH || isExpanded)
  ) {
    const highlighted = highlightedLines.value.get(originalLineIndex)
    if (highlighted) {
      processedLine = highlighted
      isHighlighted = true
    }
  }

  // 如果没有搜索词，直接返回
  if (!searchRegex.value) {
    return isHighlighted ? processedLine : displayLine
  }

  // 快速查找，避免线性搜索
  if (!searchResultsMap.value.has(currentLineNumber)) {
    return isHighlighted ? processedLine : displayLine
  }

  // 获取当前活跃搜索结果的详细信息
  const currentActiveResult = props.currentSearchIndex >= 0 ? props.searchResults[props.currentSearchIndex] : null
  const searchDisplayLine = isLongLine && !isExpanded ? displayLine : line

  // 如果是语法高亮的内容，检查是否有搜索匹配
  if (isHighlighted && searchDisplayLine.length < MAX_LINE_LENGTH) {
    const tempDiv = document.createElement('div')
    tempDiv.innerHTML = processedLine
    const textContent = tempDiv.textContent || tempDiv.innerText || ''

    // 如果纯文本中没有搜索匹配，直接返回语法高亮版本
    searchRegex.value.lastIndex = 0
    if (!searchRegex.value.test(textContent)) {
      return processedLine
    }
  }

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

// 搜索功能
const lastSearchTermRef = ref('')
const lastVisibleLinesCountRef = ref(0)

const performSearch = (term: string) => {
  if (!term || term.length < 2) {
    props.onSearchResults?.([], false)
    return
  }

  const results: Array<{ line: number; column: number; text: string; match: string }> = []
  const regex = new RegExp(term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')

  // 在可见行中搜索
  for (const { line, originalIndex } of visibleLines.value) {
    if (results.length >= MAX_SEARCH_RESULTS) break

    const searchLine = line.length > MAX_LINE_LENGTH ? line.substring(0, MAX_LINE_LENGTH) : line
    let match
    regex.lastIndex = 0

    while ((match = regex.exec(searchLine)) !== null && results.length < MAX_SEARCH_RESULTS) {
      results.push({
        line: props.startLineNumber + originalIndex,
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
    const virtualItems = virtualizer.value?.getVirtualItems() || []
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

// 事件处理
const handleLineClick = (originalLineIndex: number) => {
  const content = lines.value[originalLineIndex] || ''
  const lineNumber = props.startLineNumber + originalLineIndex
  const characters = content.length

  modalState.value = {
    isOpen: true,
    content,
    title: `第 ${lineNumber} 行内容`,
    description: `字符数: ${characters.toLocaleString()}`,
  }
}

const handleContentClick = (originalLineIndex: number, event: Event) => {
  const selection = window.getSelection()
  if (selection?.toString().length || (event.target as HTMLElement).closest('button')) {
    return
  }
  handleLineClick(originalLineIndex)
}

const closeModal = () => {
  modalState.value = { isOpen: false }
}

// 监听器
watch(() => props.searchTerm, (newTerm) => {
  // 只有搜索词真正变化时才执行搜索
  const currentVisibleCount = visibleLines.value.length
  const shouldSearch =
    newTerm !== lastSearchTermRef.value ||
    Math.abs(currentVisibleCount - lastVisibleLinesCountRef.value) > 100

  if (shouldSearch) {
    lastSearchTermRef.value = newTerm
    lastVisibleLinesCountRef.value = currentVisibleCount
    performSearch(newTerm)
  }
})

watch(() => lines.value.length, (newLength) => {
  // 自动加载
  if (newLength < 30 && props.onScrollToBottom) {
    setTimeout(props.onScrollToBottom, 100)
  }

  // 内容变化时清空高亮缓存
  highlightedLines.value.clear()
})

// 监听虚拟化器项目变化，触发语法高亮
watch(
  () => virtualizer.value?.getVirtualItems() || [],
  (virtualItems) => {
    if (shouldHighlight.value && virtualItems.length > 0) {
      highlightVisibleLines(virtualItems)
    }

    // 更新可见范围用于按需折叠计算
    if (virtualItems.length > 0) {
      const start = Math.max(0, virtualItems[0].index - 50)
      const end = Math.min(visibleLines.value.length - 1, virtualItems[virtualItems.length - 1].index + 50)

      const startOriginalIndex = visibleLines.value[start]?.originalIndex || 0
      const endOriginalIndex = visibleLines.value[end]?.originalIndex || lines.value.length - 1

      visibleRange.value = {
        start: Math.max(0, startOriginalIndex - 20),
        end: Math.min(lines.value.length - 1, endOriginalIndex + 20),
      }
    }
  },
  { deep: true }
)

// 监听主题变化，清空高亮缓存
watch(
  () => document.documentElement.classList.contains('dark'),
  () => {
    highlightedLines.value.clear()
  }
)

// 内容变化后调整滚动位置的精确恢复
watch(
  [shouldAdjustScrollAfterPrepend, scrollAdjustmentData, () => lines.value.length],
  async ([shouldAdjust, adjustData, linesLength]) => {
    if (shouldAdjust && adjustData) {
      const container = containerRef.value
      if (container) {
        const currentLinesCount = linesLength
        const actualAddedLines = currentLinesCount - adjustData.previousLinesCount

        if (actualAddedLines > 0) {
          // 计算新的虚拟行索引：原来的索引 + 新增的行数
          const newVisibleStartIndex = adjustData.visibleStartIndex + actualAddedLines

          // 使用虚拟化器精确滚动到对应位置
          await nextTick()
          virtualizer.value.scrollToIndex(newVisibleStartIndex, { align: 'start' })

          // 微调滚动位置，加上在第一个项目内的偏移
          setTimeout(() => {
            if (adjustData.scrollOffsetInFirstItem > 0) {
              container.scrollTop += adjustData.scrollOffsetInFirstItem
            }

            // 更新滚动方向跟踪，避免触发其他加载
            lastScrollTop.value = container.scrollTop
            scrollDirection.value = 'none'
            consecutiveUpScrollCount.value = 0
          }, 0)
        }
      }

      // 重置状态
      shouldAdjustScrollAfterPrepend.value = false
      scrollAdjustmentData.value = null
    }
  },
  { deep: true }
)

// 暴露给父组件的方法
const tempExpandedLineRef = ref<number | null>(null)

defineExpose({
  scrollToLine: (lineNumber: number, column?: number) => {
    // 计算目标行在原始文本中的索引
    const targetOriginalIndex = lineNumber - props.startLineNumber

    // 在可见行中找到对应的虚拟行索引
    const visibleIndex = visibleLines.value.findIndex(
      item => item.originalIndex === targetOriginalIndex
    )

    if (visibleIndex >= 0) {
      // 找到了对应的可见行，滚动到该位置
      virtualizer.value.scrollToIndex(visibleIndex, { align: 'center' })

      // 如果指定了列位置，处理横向滚动
      if (column && column > 0) {
        // 检查这行是否是长行且被折叠了
        const targetLine = lines.value[targetOriginalIndex] || ''
        const isLongLine = targetLine.length > LONG_LINE_THRESHOLD
        const isCurrentlyExpanded = expandedLongLines.value.has(targetOriginalIndex)
        const needsExpansion = isLongLine && !isCurrentlyExpanded

        // 如果需要展开，先展开
        if (needsExpansion) {
          // 收起之前临时展开的行
          if (tempExpandedLineRef.value !== null && tempExpandedLineRef.value !== targetOriginalIndex) {
            const newSet = new Set(expandedLongLines.value)
            newSet.delete(tempExpandedLineRef.value)
            expandedLongLines.value = newSet
          }

          // 展开当前行
          const newSet = new Set(expandedLongLines.value)
          newSet.add(targetOriginalIndex)
          expandedLongLines.value = newSet
          tempExpandedLineRef.value = targetOriginalIndex
        }

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
        }, needsExpansion ? 150 : 100)
      }
    } else if (targetOriginalIndex >= 0 && targetOriginalIndex < lines.value.length) {
      // 目标行存在但不在可见行列表中（可能因为代码折叠）
      // 尝试滚动到最接近的可见行
      let closestVisibleIndex = 0
      let minDistance = Infinity

      visibleLines.value.forEach((item, index) => {
        const distance = Math.abs(item.originalIndex - targetOriginalIndex)
        if (distance < minDistance) {
          minDistance = distance
          closestVisibleIndex = index
        }
      })

      virtualizer.value.scrollToIndex(closestVisibleIndex, { align: 'center' })
    }
  },
  scrollToPercentage: (percentage: number) => {
    const targetIndex = Math.floor((visibleLines.value.length - 1) * (percentage / 100))
    virtualizer.value.scrollToIndex(targetIndex, { align: 'start' })
  },
  jumpToFilePosition: (filePosition: number) => {
    let currentPosition = 0
    let targetLineIndex = 0

    for (let i = 0; i < lines.value.length; i++) {
      if (currentPosition >= filePosition) {
        targetLineIndex = i
        break
      }
      currentPosition += lines.value[i].length + 1
    }

    // 在可见行中找到对应的虚拟行索引
    const visibleIndex = visibleLines.value.findIndex(
      item => item.originalIndex === targetLineIndex
    )
    if (visibleIndex >= 0) {
      virtualizer.value.scrollToIndex(visibleIndex, { align: 'center' })
    }
  }
})

// 生命周期
onMounted(() => {
  const container = containerRef.value
  if (container) {
    container.addEventListener('scroll', handleScroll, { passive: true })
  }

  // 自动加载逻辑
  if (lines.value.length < 30 && props.onScrollToBottom) {
    setTimeout(props.onScrollToBottom, 100)
  }
})

onUnmounted(() => {
  const container = containerRef.value
  if (container) {
    container.removeEventListener('scroll', handleScroll)
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

/* 语法高亮样式 */
:deep(.shiki) {
  background: transparent !important;
  font-family: inherit !important;
}

:deep(.shiki code) {
  background: transparent !important;
  font-family: inherit !important;
}

/* 确保语法高亮的span元素显示正确 */
:deep(.shiki span) {
  font-family: inherit !important;
}
</style>