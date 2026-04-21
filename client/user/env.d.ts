/// <reference types="vite/client" />

/**
 * 环境变量类型声明
 */
interface ImportMetaEnv {
  /** 开发服务器端口 */
  readonly VITE_PORT: string
  /** 开发服务器主机 */
  readonly VITE_HOST: string
  /** 应用环境: development | production */
  readonly VITE_APP_ENV: 'development' | 'production'
  /** API 基础 URL */
  readonly VITE_API_BASE_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
