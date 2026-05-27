import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import zh from './locales/zh.json'
import ja from './locales/ja.json'

/**
 * 支持的语言列表
 */
export const supportedLocales = [
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'zh', name: '简体中文', flag: '🇨🇳' },
  { code: 'ja', name: '日本語', flag: '🇯🇵' },
] as const

/**
 * 默认语言
 */
export const defaultLocale = 'zh'

/**
 * 从本地存储获取保存的语言设置
 * @returns 语言代码
 */
function getSavedLocale(): string {
  if (typeof localStorage !== 'undefined') {
    const saved = localStorage.getItem('locale')
    if (saved && supportedLocales.some(l => l.code === saved)) {
      return saved
    }
  }
  return defaultLocale
}

/**
 * 保存语言设置到本地存储
 * @param locale - 语言代码
 */
export function saveLocale(locale: string): void {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('locale', locale)
  }
}

/**
 * 创建 i18n 实例
 */
export const i18n = createI18n({
  legacy: false,
  locale: getSavedLocale(),
  fallbackLocale: 'en',
  messages: {
    en,
    zh,
    ja,
  },
  // 禁用 linked message 功能，避免 @ 符号被解析
  flatJson: true,
})

/**
 * 切换语言
 * @param locale - 目标语言代码
 */
export function setLocale(locale: 'en' | 'zh' | 'ja'): void {
  if (supportedLocales.some(l => l.code === locale)) {
    i18n.global.locale.value = locale
    saveLocale(locale)
  }
}

/**
 * 获取当前语言
 * @returns 当前语言代码
 */
export function getCurrentLocale(): string {
  return i18n.global.locale.value
}

export default i18n
