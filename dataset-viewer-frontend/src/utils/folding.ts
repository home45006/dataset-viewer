// 代码折叠相关类型定义和工具函数

export interface FoldableRange {
  id: string
  startLine: number
  endLine: number
  type: 'function' | 'class' | 'object' | 'array' | 'block' | 'comment'
  summary: string
}

export interface VisibleLine {
  line: string
  originalIndex: number
}

/**
 * 检测可折叠的代码块
 */
export function detectFoldableRanges(
  lines: string[],
  fileName: string,
  visibleRange: { start: number; end: number }
): FoldableRange[] {
  const ranges: FoldableRange[] = []
  const language = getLanguageFromFileName(fileName)

  // 只处理可见范围内的行，提高性能
  const startLine = Math.max(0, visibleRange.start)
  const endLine = Math.min(lines.length - 1, visibleRange.end)

  // 检测不同类型的可折叠块
  detectBraceBlocks(lines, startLine, endLine, ranges, language)
  detectCommentBlocks(lines, startLine, endLine, ranges, language)
  detectFunctionBlocks(lines, startLine, endLine, ranges, language)

  return ranges
}

/**
 * 从文件名获取语言
 */
function getLanguageFromFileName(fileName: string): string {
  const extension = fileName.split('.').pop()?.toLowerCase() || ''
  const languageMap: Record<string, string> = {
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    json: 'json',
    py: 'python',
    java: 'java',
    cpp: 'cpp',
    c: 'c',
    cs: 'csharp',
    go: 'go',
    rust: 'rust',
    php: 'php',
    rb: 'ruby'
  }
  return languageMap[extension] || 'text'
}

/**
 * 检测大括号块 ({})
 */
function detectBraceBlocks(
  lines: string[],
  startLine: number,
  endLine: number,
  ranges: FoldableRange[],
  language: string
): void {
  const stack: Array<{ line: number; type: string }> = []

  for (let i = startLine; i <= endLine; i++) {
    const line = lines[i] || ''
    const trimmed = line.trim()

    // 跳过注释行
    if (isCommentLine(trimmed, language)) continue

    // 检测开始大括号
    const openBraceMatch = line.match(/[{[]/)
    if (openBraceMatch) {
      const type = getBlockType(line, language)
      stack.push({ line: i, type })
    }

    // 检测结束大括号
    const closeBraceMatch = line.match(/[}\]]/)
    if (closeBraceMatch && stack.length > 0) {
      const start = stack.pop()!
      const lineCount = i - start.line + 1

      // 只折叠超过3行的块
      if (lineCount > 3) {
        ranges.push({
          id: `${start.line}-${i}`,
          startLine: start.line,
          endLine: i,
          type: start.type as any,
          summary: getSummary(lines, start.line, i, start.type)
        })
      }
    }
  }
}

/**
 * 检测注释块
 */
function detectCommentBlocks(
  lines: string[],
  startLine: number,
  endLine: number,
  ranges: FoldableRange[],
  language: string
): void {
  let blockStart = -1

  for (let i = startLine; i <= endLine; i++) {
    const line = lines[i] || ''
    const trimmed = line.trim()

    if (isCommentLine(trimmed, language)) {
      if (blockStart === -1) {
        blockStart = i
      }
    } else {
      if (blockStart !== -1 && i - blockStart > 2) {
        ranges.push({
          id: `comment-${blockStart}-${i - 1}`,
          startLine: blockStart,
          endLine: i - 1,
          type: 'comment',
          summary: `Comment block (${i - blockStart} lines)`
        })
      }
      blockStart = -1
    }
  }

  // 处理文件末尾的注释块
  if (blockStart !== -1 && endLine - blockStart > 2) {
    ranges.push({
      id: `comment-${blockStart}-${endLine}`,
      startLine: blockStart,
      endLine: endLine,
      type: 'comment',
      summary: `Comment block (${endLine - blockStart + 1} lines)`
    })
  }
}

/**
 * 检测函数块
 */
function detectFunctionBlocks(
  lines: string[],
  startLine: number,
  endLine: number,
  ranges: FoldableRange[],
  language: string
): void {
  const functionPatterns: Record<string, RegExp[]> = {
    javascript: [
      /function\s+\w+\s*\(/,
      /const\s+\w+\s*=\s*\(/,
      /\w+\s*:\s*function\s*\(/,
      /\w+\s*\([^)]*\)\s*=>/
    ],
    typescript: [
      /function\s+\w+\s*\(/,
      /const\s+\w+\s*=\s*\(/,
      /\w+\s*:\s*function\s*\(/,
      /\w+\s*\([^)]*\)\s*=>/
    ],
    python: [
      /def\s+\w+\s*\(/,
      /class\s+\w+/
    ],
    java: [
      /(public|private|protected)?\s*(static\s+)?[\w<>]+\s+\w+\s*\(/,
      /(public|private|protected)?\s*class\s+\w+/
    ]
  }

  const patterns = functionPatterns[language] || []

  for (let i = startLine; i <= endLine; i++) {
    const line = lines[i] || ''

    for (const pattern of patterns) {
      if (pattern.test(line)) {
        // 寻找对应的结束行
        const endLineIndex = findBlockEnd(lines, i, language)
        if (endLineIndex > i + 2) {
          ranges.push({
            id: `function-${i}-${endLineIndex}`,
            startLine: i,
            endLine: endLineIndex,
            type: 'function',
            summary: getFunctionSummary(line)
          })
        }
        break
      }
    }
  }
}

/**
 * 判断是否是注释行
 */
function isCommentLine(line: string, language: string): boolean {
  const commentPatterns: Record<string, RegExp[]> = {
    javascript: [/^\/\//, /^\/\*/, /^\*/],
    typescript: [/^\/\//, /^\/\*/, /^\*/],
    python: [/^#/],
    java: [/^\/\//, /^\/\*/, /^\*/],
    cpp: [/^\/\//, /^\/\*/, /^\*/],
    c: [/^\/\//, /^\/\*/, /^\*/]
  }

  const patterns = commentPatterns[language] || [/^\/\//, /^#/]
  return patterns.some(pattern => pattern.test(line))
}

/**
 * 获取块类型
 */
function getBlockType(line: string, language: string): string {
  if (line.includes('function') || line.includes('def ')) return 'function'
  if (line.includes('class ')) return 'class'
  if (line.includes('[')) return 'array'
  if (line.includes('{') && (line.includes(':') || line.includes('='))) return 'object'
  return 'block'
}

/**
 * 获取摘要信息
 */
function getSummary(lines: string[], startLine: number, endLine: number, type: string): string {
  const lineCount = endLine - startLine + 1
  const firstLine = lines[startLine]?.trim() || ''

  if (type === 'function') {
    return getFunctionSummary(firstLine)
  }

  if (type === 'class') {
    const match = firstLine.match(/class\s+(\w+)/)
    return match ? `class ${match[1]} (${lineCount} lines)` : `Class block (${lineCount} lines)`
  }

  return `${type} (${lineCount} lines)`
}

/**
 * 获取函数摘要
 */
function getFunctionSummary(line: string): string {
  // JavaScript/TypeScript 函数
  let match = line.match(/function\s+(\w+)\s*\(([^)]*)\)/)
  if (match) {
    return `function ${match[1]}(${match[2]})`
  }

  // 箭头函数
  match = line.match(/const\s+(\w+)\s*=\s*\(([^)]*)\)\s*=>/)
  if (match) {
    return `${match[1]}(${match[2]}) =>`
  }

  // Python 函数
  match = line.match(/def\s+(\w+)\s*\(([^)]*)\)/)
  if (match) {
    return `def ${match[1]}(${match[2]})`
  }

  return line.substring(0, 50) + (line.length > 50 ? '...' : '')
}

/**
 * 寻找块的结束行
 */
function findBlockEnd(lines: string[], startLine: number, language: string): number {
  const openChars = ['{', '(', '[']
  const closeChars = ['}', ')', ']']
  const stack: string[] = []

  for (let i = startLine; i < lines.length; i++) {
    const line = lines[i] || ''

    for (const char of line) {
      if (openChars.includes(char)) {
        stack.push(char)
      } else if (closeChars.includes(char)) {
        if (stack.length > 0) {
          stack.pop()
        }
        if (stack.length === 0 && i > startLine) {
          return i
        }
      }
    }
  }

  return startLine
}

/**
 * 生成可见行列表（考虑折叠状态）
 */
export function generateVisibleLines(
  lines: string[],
  collapsedRanges: Set<string>
): VisibleLine[] {
  const visibleLines: VisibleLine[] = []
  const collapsedLines = new Set<number>()

  // 收集所有被折叠的行号
  collapsedRanges.forEach(rangeId => {
    const [start, end] = rangeId.split('-').map(Number)
    if (!isNaN(start) && !isNaN(end)) {
      for (let i = start + 1; i <= end; i++) {
        collapsedLines.add(i)
      }
    }
  })

  // 生成可见行列表
  lines.forEach((line, index) => {
    if (!collapsedLines.has(index)) {
      visibleLines.push({
        line,
        originalIndex: index
      })
    }
  })

  return visibleLines
}