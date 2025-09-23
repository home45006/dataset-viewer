import { ref, computed, watch } from 'vue'
import { detectFoldableRanges, generateVisibleLines, type FoldableRange, type VisibleLine } from '../utils/folding'

interface UseFoldingLogicOptions {
  lines: string[]
  fileName: string
  visibleRange: { start: number; end: number }
}

export function useFoldingLogic(options: UseFoldingLogicOptions) {
  const collapsedRanges = ref<Set<string>>(new Set())
  const cachedRanges = ref<Map<string, FoldableRange[]>>(new Map())

  // 计算是否支持折叠
  const supportsFolding = computed(() => {
    const extension = options.fileName.split('.').pop()?.toLowerCase() || ''
    const supportedExtensions = ['js', 'jsx', 'ts', 'tsx', 'json', 'py', 'java', 'cpp', 'c', 'cs', 'go', 'rust', 'php', 'rb']
    return supportedExtensions.includes(extension)
  })

  // 计算可折叠范围
  const foldableRanges = computed(() => {
    if (!supportsFolding.value || options.lines.length === 0) {
      return []
    }

    // 生成缓存键
    const cacheKey = `${options.fileName}-${options.visibleRange.start}-${options.visibleRange.end}-${options.lines.length}`

    // 检查缓存
    if (cachedRanges.value.has(cacheKey)) {
      return cachedRanges.value.get(cacheKey)!
    }

    // 计算新的折叠范围
    const ranges = detectFoldableRanges(options.lines, options.fileName, options.visibleRange)

    // 更新缓存
    cachedRanges.value.set(cacheKey, ranges)

    // 限制缓存大小
    if (cachedRanges.value.size > 10) {
      const firstKey = cachedRanges.value.keys().next().value
      cachedRanges.value.delete(firstKey)
    }

    return ranges
  })

  // 计算可见行
  const visibleLines = computed(() => {
    return generateVisibleLines(options.lines, collapsedRanges.value)
  })

  // 获取指定行的可折叠范围
  const getFoldableRangeAtLine = (lineIndex: number): FoldableRange | null => {
    return foldableRanges.value.find(range => range.startLine === lineIndex) || null
  }

  // 切换折叠状态
  const toggleFoldingRange = (rangeId: string) => {
    const newCollapsed = new Set(collapsedRanges.value)
    if (newCollapsed.has(rangeId)) {
      newCollapsed.delete(rangeId)
    } else {
      newCollapsed.add(rangeId)
    }
    collapsedRanges.value = newCollapsed
  }

  // 展开所有折叠
  const expandAll = () => {
    collapsedRanges.value = new Set()
  }

  // 折叠所有可折叠块
  const collapseAll = () => {
    const allRangeIds = foldableRanges.value.map(range => range.id)
    collapsedRanges.value = new Set(allRangeIds)
  }

  // 监听内容变化，清理缓存
  watch([() => options.lines.length, () => options.fileName], () => {
    cachedRanges.value.clear()
    collapsedRanges.value.clear()
  })

  return {
    supportsFolding,
    foldableRanges,
    collapsedRanges,
    visibleLines,
    getFoldableRangeAtLine,
    toggleFoldingRange,
    expandAll,
    collapseAll
  }
}