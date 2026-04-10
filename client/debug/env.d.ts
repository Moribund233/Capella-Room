/// <reference types="vite/client" />

// 环境变量类型声明
interface ImportMetaEnv {
  readonly VITE_PORT: string
  readonly VITE_BACKEND_URL: string
  readonly VITE_WS_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

// 全局常量（由 vite.config.ts define 定义）
declare const __BACKEND_URL__: string
