import { createI18n } from 'vue-i18n'

// 导入语言文件
import zh from './locales/zh.json'
import en from './locales/en.json'

const messages = {
  zh,
  en,
}

// 检测浏览器语言
const getLocale = () => {
  const saved = localStorage.getItem('locale')
  if (saved && messages[saved as keyof typeof messages]) {
    return saved
  }
  
  const browserLang = navigator.language.split('-')[0]
  return messages[browserLang as keyof typeof messages] ? browserLang : 'en'
}

const i18n = createI18n({
  legacy: false,
  locale: getLocale(),
  fallbackLocale: 'en',
  messages,
})

export default i18n