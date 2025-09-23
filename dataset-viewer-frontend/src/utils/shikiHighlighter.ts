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
  log: 'text',
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

export function getFileLanguage(filename: string | undefined): string {
  if (!filename) return 'text'

  const extension = filename.split('.').pop()?.toLowerCase()
  if (!extension) return 'text'

  return LANGUAGE_MAP[extension] || 'text'
}

export function isCodeFile(filename: string | undefined): boolean {
  if (!filename) return false

  const language = getFileLanguage(filename)
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
    const language = getFileLanguage(filename)

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