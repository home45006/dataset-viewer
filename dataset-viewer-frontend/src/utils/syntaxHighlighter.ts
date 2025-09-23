// 简化的语法高亮器，不依赖WebAssembly或复杂的外部库

interface HighlighterCache {
  [key: string]: string
}

// 高亮器缓存
const highlighterCache: HighlighterCache = {}

// 支持的语言映射
const languageMap: Record<string, string> = {
  // JavaScript 相关
  js: 'javascript',
  jsx: 'javascript',
  javascript: 'javascript',
  ts: 'typescript',
  tsx: 'typescript',
  typescript: 'typescript',

  // Web 相关
  html: 'html',
  htm: 'html',
  css: 'css',
  scss: 'css',
  sass: 'css',
  less: 'css',

  // Python
  py: 'python',
  python: 'python',
  pyx: 'python',
  pyw: 'python',

  // Java 相关
  java: 'java',
  kt: 'kotlin',
  kotlin: 'kotlin',
  scala: 'scala',
  groovy: 'java',

  // C 相关
  c: 'c',
  cpp: 'cpp',
  cc: 'cpp',
  cxx: 'cpp',
  'c++': 'cpp',
  h: 'c',
  hpp: 'cpp',

  // C#
  cs: 'csharp',
  csharp: 'csharp',

  // 配置文件
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  xml: 'xml',
  toml: 'toml',
  ini: 'ini',

  // Shell 相关
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  fish: 'bash',

  // 数据库
  sql: 'sql',

  // 其他语言
  go: 'go',
  rust: 'rust',
  rs: 'rust',
  php: 'php',
  rb: 'ruby',
  ruby: 'ruby',
  swift: 'swift',
  dart: 'dart',
  lua: 'lua',
  r: 'r',
  matlab: 'matlab',

  // 标记语言
  md: 'markdown',
  markdown: 'markdown',
  tex: 'latex',
  latex: 'latex',

  // 数据格式
  csv: 'text',
  tsv: 'text',

  // 配置文件
  dockerfile: 'dockerfile',
  makefile: 'makefile',
  gitignore: 'text',
  gitattributes: 'text',

  // Web相关
  vue: 'javascript',
  svelte: 'javascript',

  // 其他语言
  clj: 'clojure',
  clojure: 'clojure',
  ex: 'elixir',
  exs: 'elixir',
  elixir: 'elixir',
  erl: 'erlang',
  erlang: 'erlang',

  // 其他
  txt: 'text',
  text: 'text',
  log: 'text',
  conf: 'text',
  config: 'text'
}

/**
 * 从文件名获取语言类型
 */
export function getLanguageFromFileName(fileName: string): string {
  const extension = fileName.split('.').pop()?.toLowerCase() || ''
  return languageMap[extension] || 'text'
}

/**
 * 检查语言是否被支持
 */
export function isLanguageSupported(language: string): boolean {
  const supportedLanguages = [
    'javascript', 'typescript', 'html', 'css', 'python', 'java', 'kotlin', 'scala',
    'c', 'cpp', 'csharp', 'json', 'yaml', 'xml', 'toml', 'ini', 'bash', 'sql',
    'go', 'rust', 'php', 'ruby', 'swift', 'dart', 'lua', 'r', 'matlab',
    'markdown', 'latex', 'dockerfile', 'makefile', 'clojure', 'elixir', 'erlang'
  ]
  return supportedLanguages.includes(language) && language !== 'text'
}

/**
 * 获取语言特定的颜色配置
 */
function getColors(theme: 'light' | 'dark') {
  const isDark = theme === 'dark'

  return {
    keyword: isDark ? '#569cd6' : '#0000ff',
    string: isDark ? '#ce9178' : '#a31515',
    comment: isDark ? '#6a9955' : '#008000',
    number: isDark ? '#b5cea8' : '#09885a',
    function: isDark ? '#dcdcaa' : '#795e26',
    type: isDark ? '#4ec9b0' : '#267f99',
    constant: isDark ? '#4fc1ff' : '#0070c1',
    operator: isDark ? '#d4d4d4' : '#000000',
    punctuation: isDark ? '#d4d4d4' : '#000000'
  }
}

/**
 * 获取语言特定的关键字和模式
 */
function getLanguagePatterns(language: string) {
  const patterns: Record<string, any> = {
    javascript: {
      keywords: ['function', 'const', 'let', 'var', 'if', 'else', 'for', 'while', 'return', 'class', 'import', 'export', 'async', 'await', 'try', 'catch', 'finally', 'throw', 'new', 'this', 'super', 'extends', 'implements', 'interface', 'type', 'enum'],
      types: ['string', 'number', 'boolean', 'object', 'undefined', 'null', 'Array', 'Object', 'Function', 'Promise'],
      constants: ['true', 'false', 'null', 'undefined', 'NaN', 'Infinity'],
      operators: ['===', '!==', '==', '!=', '<=', '>=', '&&', '||', '++', '--', '=>']
    },
    typescript: {
      keywords: ['function', 'const', 'let', 'var', 'if', 'else', 'for', 'while', 'return', 'class', 'import', 'export', 'async', 'await', 'try', 'catch', 'finally', 'throw', 'new', 'this', 'super', 'extends', 'implements', 'interface', 'type', 'enum', 'public', 'private', 'protected', 'readonly', 'static'],
      types: ['string', 'number', 'boolean', 'object', 'undefined', 'null', 'Array', 'Object', 'Function', 'Promise', 'any', 'void', 'never'],
      constants: ['true', 'false', 'null', 'undefined', 'NaN', 'Infinity'],
      operators: ['===', '!==', '==', '!=', '<=', '>=', '&&', '||', '++', '--', '=>']
    },
    python: {
      keywords: ['def', 'class', 'if', 'else', 'elif', 'for', 'while', 'return', 'import', 'from', 'try', 'except', 'with', 'as', 'pass', 'break', 'continue', 'lambda', 'yield', 'global', 'nonlocal', 'assert', 'del', 'raise', 'finally', 'and', 'or', 'not', 'in', 'is'],
      types: ['int', 'float', 'str', 'bool', 'list', 'dict', 'tuple', 'set', 'None'],
      constants: ['True', 'False', 'None'],
      operators: ['==', '!=', '<=', '>=', 'and', 'or', 'not', 'in', 'is']
    },
    java: {
      keywords: ['public', 'private', 'protected', 'static', 'final', 'abstract', 'class', 'interface', 'extends', 'implements', 'if', 'else', 'for', 'while', 'return', 'try', 'catch', 'finally', 'throw', 'throws', 'new', 'this', 'super', 'import', 'package'],
      types: ['int', 'float', 'double', 'boolean', 'char', 'byte', 'short', 'long', 'String', 'Object', 'void'],
      constants: ['true', 'false', 'null'],
      operators: ['==', '!=', '<=', '>=', '&&', '||', '++', '--']
    },
    go: {
      keywords: ['func', 'var', 'const', 'type', 'struct', 'interface', 'if', 'else', 'for', 'while', 'return', 'import', 'package', 'go', 'defer', 'chan', 'select', 'case', 'default', 'fallthrough', 'break', 'continue'],
      types: ['int', 'int8', 'int16', 'int32', 'int64', 'uint', 'uint8', 'uint16', 'uint32', 'uint64', 'float32', 'float64', 'bool', 'string', 'byte', 'rune'],
      constants: ['true', 'false', 'nil', 'iota'],
      operators: ['==', '!=', '<=', '>=', '&&', '||', '++', '--', ':=', '<-']
    },
    rust: {
      keywords: ['fn', 'let', 'mut', 'const', 'static', 'struct', 'enum', 'trait', 'impl', 'if', 'else', 'match', 'for', 'while', 'loop', 'return', 'use', 'mod', 'pub', 'crate', 'self', 'super', 'where', 'unsafe', 'async', 'await'],
      types: ['i8', 'i16', 'i32', 'i64', 'u8', 'u16', 'u32', 'u64', 'f32', 'f64', 'bool', 'char', 'str', 'String', 'Vec', 'Option', 'Result'],
      constants: ['true', 'false', 'None', 'Some', 'Ok', 'Err'],
      operators: ['==', '!=', '<=', '>=', '&&', '||', '=>', '..', '...']
    },
    clojure: {
      keywords: ['defn', 'defn-', 'def', 'let', 'if', 'cond', 'when', 'when-not', 'case', 'loop', 'recur', 'fn', 'defmacro', 'ns', 'require', 'use', 'import', 'gen-class', 'proxy', 'doto', 'new'],
      types: ['String', 'Integer', 'Long', 'Float', 'Double', 'Boolean', 'Character', 'Keyword', 'Symbol'],
      constants: ['true', 'false', 'nil'],
      operators: ['=', '==', 'not=', '<', '>', '<=', '>=', 'and', 'or', 'not']
    },
    elixir: {
      keywords: ['defmodule', 'def', 'defp', 'defmacro', 'defstruct', 'if', 'unless', 'cond', 'case', 'with', 'for', 'try', 'catch', 'rescue', 'after', 'else', 'end', 'do', 'when', 'fn', 'import', 'require', 'alias', 'use'],
      types: ['atom', 'binary', 'bitstring', 'boolean', 'float', 'function', 'integer', 'list', 'map', 'nil', 'pid', 'port', 'reference', 'tuple'],
      constants: ['true', 'false', 'nil'],
      operators: ['==', '!=', '===', '!==', '<', '>', '<=', '>=', 'and', 'or', 'not', '++', '--', '<>', '|>', '<-', '=>']
    },
    erlang: {
      keywords: ['after', 'and', 'andalso', 'band', 'begin', 'bnot', 'bor', 'bsl', 'bsr', 'bxor', 'case', 'catch', 'cond', 'div', 'end', 'fun', 'if', 'let', 'not', 'of', 'or', 'orelse', 'query', 'receive', 'rem', 'try', 'when', 'xor'],
      types: ['atom', 'binary', 'bitstring', 'boolean', 'float', 'function', 'integer', 'list', 'map', 'number', 'pid', 'port', 'reference', 'tuple'],
      constants: ['true', 'false', 'undefined'],
      operators: ['==', '/=', '=:=', '=/=', '<', '>', '=<', '>=', 'and', 'or', 'not', '++', '--', '!', '?']
    },
    json: {
      keywords: [], // JSON 没有关键字
      types: [],
      constants: ['true', 'false', 'null'],
      operators: []
    }
  }

  return patterns[language] || patterns.javascript
}

/**
 * 高亮单行代码
 */
export async function highlightLine(
  code: string,
  language: string,
  theme: 'light' | 'dark' = 'light'
): Promise<string> {
  try {
    if (!isLanguageSupported(language) || !code.trim()) {
      return code
    }

    // 生成缓存键
    const cacheKey = `${language}-${theme}-${code}`
    if (highlighterCache[cacheKey]) {
      return highlighterCache[cacheKey]
    }

    const colors = getColors(theme)
    const patterns = getLanguagePatterns(language)
    let highlighted = code

    // HTML转义
    highlighted = highlighted
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')

    // JSON 特殊处理
    if (language === 'json') {
      // JSON 属性名高亮（双引号后跟冒号）
      highlighted = highlighted.replace(
        /"([^"\\]|\\.)*"(\s*:)/g,
        `<span style="color:${colors.type}; font-weight: bold">"$1"</span>$2`
      )

      // JSON 字符串值高亮（不是属性名的字符串）
      highlighted = highlighted.replace(
        /"([^"\\]|\\.)*"(?!\s*:)/g,
        `<span style="color:${colors.string}">$&</span>`
      )

      // JSON 数值高亮
      highlighted = highlighted.replace(
        /\b-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?\b/g,
        `<span style="color:${colors.number}">$&</span>`
      )

      // JSON 布尔值和null高亮
      highlighted = highlighted.replace(
        /\b(true|false|null)\b/g,
        `<span style="color:${colors.constant}">$1</span>`
      )

      // JSON 结构符号高亮
      highlighted = highlighted.replace(
        /([{}[\],:])/g,
        `<span style="color:${colors.operator}">$1</span>`
      )
    } else {
      // 其他语言的字符串高亮 (优先级最高，先处理)
      highlighted = highlighted.replace(
        /(["'`])((?:\\.|(?!\1)[^\\])*?)\1/g,
        `<span style="color:${colors.string}">$&</span>`
      )
    }

    // 对于非JSON语言，应用常规的高亮规则
    if (language !== 'json') {
      // 注释高亮
      if (language === 'python') {
        highlighted = highlighted.replace(/(#.*$)/gm, `<span style="color:${colors.comment}">$1</span>`)
      } else {
        highlighted = highlighted.replace(
          /(\/\/.*$|\/\*[\s\S]*?\*\/)/gm,
          `<span style="color:${colors.comment}">$1</span>`
        )
      }

      // 数字高亮
      highlighted = highlighted.replace(
        /\b(\d+\.?\d*|\.\d+)\b/g,
        `<span style="color:${colors.number}">$1</span>`
      )

      // 关键字高亮
      if (patterns.keywords) {
        patterns.keywords.forEach((keyword: string) => {
          const regex = new RegExp(`\\b${keyword}\\b`, 'g')
          highlighted = highlighted.replace(regex, `<span style="color:${colors.keyword}">$&</span>`)
        })
      }

      // 类型高亮
      if (patterns.types) {
        patterns.types.forEach((type: string) => {
          const regex = new RegExp(`\\b${type}\\b`, 'g')
          highlighted = highlighted.replace(regex, `<span style="color:${colors.type}">$&</span>`)
        })
      }

      // 常量高亮
      if (patterns.constants) {
        patterns.constants.forEach((constant: string) => {
          const regex = new RegExp(`\\b${constant}\\b`, 'g')
          highlighted = highlighted.replace(regex, `<span style="color:${colors.constant}">$&</span>`)
        })
      }

      // 函数调用高亮
      highlighted = highlighted.replace(
        /\b([a-zA-Z_$][a-zA-Z0-9_$]*)\s*(?=\()/g,
        `<span style="color:${colors.function}">$1</span>`
      )
    }

    // 缓存结果
    highlighterCache[cacheKey] = highlighted
    return highlighted

  } catch (error) {
    console.warn('Syntax highlighting failed:', error)
    return code
  }
}

/**
 * 高亮多行代码
 */
export async function highlightCode(
  code: string,
  language: string,
  theme: 'light' | 'dark' = 'light'
): Promise<string> {
  try {
    if (!isLanguageSupported(language) || !code.trim()) {
      return `<pre><code>${code}</code></pre>`
    }

    const lines = code.split('\n')
    const highlightedLines = await Promise.all(
      lines.map(line => highlightLine(line, language, theme))
    )

    return `<pre><code>${highlightedLines.join('\n')}</code></pre>`
  } catch (error) {
    console.warn('Syntax highlighting failed:', error)
    return `<pre><code>${code}</code></pre>`
  }
}

/**
 * 预加载常用语言的高亮器（空实现，保持API兼容）
 */
export async function preloadHighlighters(): Promise<void> {
  // 不需要预加载，因为我们使用的是同步的正则表达式高亮
}

/**
 * 清理高亮器缓存
 */
export function clearHighlighterCache(): void {
  Object.keys(highlighterCache).forEach(key => {
    delete highlighterCache[key]
  })
}