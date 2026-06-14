import { fileURLToPath, URL } from 'node:url'
import { readFileSync, existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'

import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'

// =============================================================================
// 读取 Capella Room 安全入口哈希
// =============================================================================
function getSecureEntryBase(): string {
  const configPath = join(homedir(), '.capella-secure-entry.conf')
  if (existsSync(configPath)) {
    const hash = readFileSync(configPath, 'utf-8').trim()
    if (hash) {
      return `/${hash}/`
    }
  }
  // 无配置时默认根路径（开发环境）
  return '/'
}

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // 加载环境变量
  const env = loadEnv(mode, process.cwd(), '')

  // 构建时自动读取安全入口哈希作为 base path
  const base = env.VITE_BASE_PATH || getSecureEntryBase()

  return {
    // 生产构建时使用安全入口路径作为 base
    base,
    plugins: [
      vue(),
      vueJsx(),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      },
    },
    server: {
      port: parseInt(env.VITE_PORT),
      host: env.VITE_HOST,
      open: env.VITE_OPEN_BROWSER,
    },
  }
})
