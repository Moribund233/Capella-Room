import { fileURLToPath, URL } from 'node:url'

import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')

  return {
    plugins: [
      vue(),
      vueJsx(),
      vueDevTools(),
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      },
    },
    server: {
      port: Number(env.VITE_SERVER_PORT),
      proxy: {
        '/api': {
          target: env.VITE_PROXY_API_TARGET,
          changeOrigin: true,
        },
        '/health': {
          target: env.VITE_PROXY_API_TARGET,
          changeOrigin: true,
        },
        '/ws': {
          target: env.VITE_PROXY_WS_TARGET,
          ws: true,
          changeOrigin: true,
        },
      },
    },
  }
})
