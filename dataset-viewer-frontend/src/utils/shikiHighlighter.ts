import { createHighlighter, type Highlighter, bundledLanguages, bundledThemes } from 'shiki'

let highlighterInstance: Highlighter | null = null
let isInitializing = false

const LANGUAGE_MAP: Record<string, string> = {
  ts: 'typescript',
  js: 'javascript',
  jsx: 'javascript',
  tsx: 'typescript',
  py: 'python',
  rb: 'ruby',
  php: 'php',
  java: 'java',
  cpp: 'cpp',
  c: 'c',
  cs: 'csharp',
  go: 'go',
  rs: 'rust',
  kt: 'kotlin',
  swift: 'swift',
  scala: 'scala',
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  fish: 'bash',
  ps1: 'powershell',
  cmd: 'batch',
  bat: 'batch',
  html: 'html',
  htm: 'html',
  xml: 'xml',
  css: 'css',
  scss: 'scss',
  sass: 'sass',
  less: 'less',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'toml',
  ini: 'ini',
  cfg: 'ini',
  conf: 'ini',
  md: 'markdown',
  markdown: 'markdown',
  tex: 'latex',
  r: 'r',
  R: 'r',
  m: 'matlab',
  pl: 'perl',
  lua: 'lua',
  vim: 'vim',
  dockerfile: 'dockerfile',
  sql: 'sql',
  graphql: 'graphql',
  proto: 'protobuf',
  thrift: 'thrift',
  avro: 'json',
  parquet: 'text',
  jsonl: 'json',
  ndjson: 'json',
  csv: 'text',
  tsv: 'text',
  txt: 'text',
  log: 'log',
  diff: 'diff',
  patch: 'diff',
  gitignore: 'text',
  gitattributes: 'text',
  editorconfig: 'ini',
  prettierrc: 'json',
  eslintrc: 'json',
  babelrc: 'json',
  npmrc: 'ini',
  yarnrc: 'ini',
  gemfile: 'ruby',
  gemspec: 'ruby',
  rakefile: 'ruby',
  makefile: 'makefile',
  cmake: 'cmake',
  vagrantfile: 'ruby',
  procfile: 'text',
  license: 'text',
  readme: 'markdown',
  changelog: 'markdown',
  contributing: 'markdown',
  authors: 'text',
  contributors: 'text',
  notice: 'text',
  copying: 'text',
  install: 'text',
  news: 'text',
  todo: 'text',
  bugs: 'text',
  acknowledgments: 'text',
  thanks: 'text',
  version: 'text',
  buildinfo: 'text',
  version_info: 'text'
}

async function initializeHighlighter(): Promise<void> {
  if (highlighterInstance || isInitializing) {
    return
  }

  isInitializing = true
  try {
    // Get unique languages that are available in bundled languages
    const languages = Object.values(LANGUAGE_MAP).filter(
      (lang, index, self) => self.indexOf(lang) === index && lang in bundledLanguages
    )

    highlighterInstance = await createHighlighter({
      themes: ['github-light', 'github-dark'],
      langs: languages,
    })
  } catch (error) {
    console.error('Failed to initialize syntax highlighter:', error)
    highlighterInstance = null
  }
  isInitializing = false
}

export function getFileLanguage(filename: string | undefined, content?: string): string {
  if (!filename) return 'text'

  const lowerFilename = filename.toLowerCase()
  const extension = filename.split('.').pop()?.toLowerCase()

  // 如果有扩展名，优先根据扩展名判断
  if (extension && extension !== lowerFilename) {
    const mappedLanguage = LANGUAGE_MAP[extension]
    if (mappedLanguage) return mappedLanguage
  }

  // 处理无扩展名的文件：根据文件名模式识别
  const filenamePatterns: Record<string, string> = {
    // 日志文件模式
    'access': 'log',
    'error': 'log',
    'debug': 'log',
    'application': 'log',
    'system': 'log',
    'audit': 'log',
    'trace': 'log',
    'stderr': 'log',
    'stdout': 'log',
    'console': 'log',
    'syslog': 'log',
    'messages': 'log',
    'kern': 'log',
    'auth': 'log',
    'mail': 'log',
    'cron': 'log',
    'daemon': 'log',
    'user': 'log',
    'local0': 'log',
    'local1': 'log',
    'local2': 'log',
    'local3': 'log',
    'local4': 'log',
    'local5': 'log',
    'local6': 'log',
    'local7': 'log',
    // 配置文件
    'dockerfile': 'dockerfile',
    'makefile': 'makefile',
    'gemfile': 'ruby',
    'rakefile': 'ruby',
    'vagrantfile': 'ruby',
    'procfile': 'text',
    'license': 'text',
    'readme': 'markdown',
    'changelog': 'markdown',
    'contributing': 'markdown',
    // 其他常见无扩展名文件
    'hosts': 'text',
    'passwd': 'text',
    'group': 'text',
    'shadow': 'text',
    'fstab': 'text',
    'mtab': 'text',
    'bashrc': 'bash',
    'zshrc': 'bash',
    'profile': 'bash',
    'vimrc': 'vim',
    'gitignore': 'text',
    'gitattributes': 'text'
  }

  // 检查完整文件名匹配
  if (filenamePatterns[lowerFilename]) {
    return filenamePatterns[lowerFilename]
  }

  // 检查文件名包含的关键词
  for (const [pattern, lang] of Object.entries(filenamePatterns)) {
    if (lowerFilename.includes(pattern)) {
      return lang
    }
  }

  // 如果提供了内容，尝试根据内容特征判断
  if (content) {
    return detectLanguageByContent(content, lowerFilename)
  }

  return 'text'
}

// 根据文件内容特征检测语言类型
function detectLanguageByContent(content: string, filename: string): string {
  const lines = content.split('\n').slice(0, 20) // 只检查前20行
  const firstFewLines = lines.join('\n').toLowerCase()

  // 日志文件特征检测
  const logPatterns = [
    /\d{4}[-/]\d{2}[-/]\d{2}/, // 日期格式 2024-01-01 或 2024/01/01
    /\d{2}:\d{2}:\d{2}/, // 时间格式 12:34:56
    /\b(info|error|warn|debug|trace|fatal)\b/i, // 日志级别
    /\[\d{4}-\d{2}-\d{2}/, // [2024-01-01 格式
    /^\d+\s+\w+\s+\d+\s+\d{2}:\d{2}:\d{2}/, // syslog 格式
    /(get|post|put|delete)\s+\/\w+.*\s+\d{3}\s+\d+/i, // HTTP 访问日志
    /\d+\.\d+\.\d+\.\d+/, // IP 地址
    /\b(started|finished|completed|failed|success|exception)\b/i // 常见日志动词
  ]

  let logScore = 0
  for (const pattern of logPatterns) {
    if (pattern.test(firstFewLines)) {
      logScore++
    }
  }

  // 如果匹配多个日志模式，认为是日志文件
  if (logScore >= 2) {
    return 'log'
  }

  // JSON 格式检测
  if (firstFewLines.trim().startsWith('{') || firstFewLines.trim().startsWith('[')) {
    try {
      JSON.parse(content.trim())
      return 'json'
    } catch (e) {
      // 不是有效的 JSON
    }
  }

  // XML 格式检测
  if (firstFewLines.includes('<?xml') || firstFewLines.includes('<html') ||
      (firstFewLines.includes('<') && firstFewLines.includes('>'))) {
    return 'xml'
  }

  // Shell 脚本检测
  if (firstFewLines.startsWith('#!') &&
      (firstFewLines.includes('/bin/bash') || firstFewLines.includes('/bin/sh') ||
       firstFewLines.includes('/usr/bin/env bash'))) {
    return 'bash'
  }

  // Python 脚本检测
  if (firstFewLines.startsWith('#!') && firstFewLines.includes('python')) {
    return 'python'
  }

  return 'text'
}

export function isCodeFile(filename: string | undefined, content?: string): boolean {
  if (!filename) return false

  const language = getFileLanguage(filename, content)
  return language !== 'text'
}

export async function highlightCode(code: string, filename: string | undefined): Promise<string> {
  if (!highlighterInstance) {
    await initializeHighlighter()
  }

  if (!highlighterInstance) {
    // Fallback to plain text if highlighter failed to initialize
    return `<pre class="shiki">${escapeHtml(code)}</pre>`
  }

  try {
    // 传递文件内容以便更好地检测语言类型
    const language = getFileLanguage(filename, code)

    // Check if the language is supported
    const loadedLanguages = highlighterInstance.getLoadedLanguages()
    const effectiveLanguage = loadedLanguages.includes(language as any) ? language : 'text'

    // Use dark theme for now (you can make this configurable)
    const highlighted = highlighterInstance.codeToHtml(code, {
      lang: effectiveLanguage,
      theme: 'github-dark'
    })

    return highlighted
  } catch (error) {
    console.error('Error highlighting code:', error)
    // Fallback to plain text
    return `<pre class="shiki">${escapeHtml(code)}</pre>`
  }
}

function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

// Initialize the highlighter on module load
initializeHighlighter().catch(console.error)