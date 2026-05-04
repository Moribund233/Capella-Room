import { defineStore } from 'pinia'
import { ref } from 'vue'
import { configApi } from '@/api/config'
import { useConnectivityStore } from './connectivity'
import type { ClientConfig } from '@/types/config'

/** 默认配置（与 server config.toml 默认值一致） */
const DEFAULT_CONFIG: ClientConfig = {
  websocket: {
    heartbeat_interval_secs: 30,
    heartbeat_timeout_secs: 90,
    auth_timeout_secs: 30,
  },
  reconnect: {
    base_delay_ms: 1000,
    max_delay_ms: 30000,
    max_attempts: 5,
    multiplier: 2,
  },
  upload: {
    max_file_size: 10485760,
    max_file_size_human: '10 MB',
  },
  system: {
    name: 'Seredeli Room',
    version: '1.0.0',
    maintenance_mode: false,
    maintenance_message: '',
  },
  monitor: {
    refresh_interval_secs: 30,
  },
}

export const useConfigStore = defineStore('config', () => {
  const config = ref<ClientConfig>({ ...DEFAULT_CONFIG })
  const loaded = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchConfig() {
    if (loaded.value || loading.value) return

    loading.value = true
    error.value = null

    try {
      const res = await configApi.getClientConfig()
      if (res.success && res.data) {
        config.value = res.data
        loaded.value = true
      }
    } catch (err) {
      console.warn('[Config] Failed to fetch server config:', err)

      // 配置获取失败 → 探活服务器
      const connectivity = useConnectivityStore()
      const status = await connectivity.probeServer()

      if (status === 'unreachable') {
        // 服务器不可达 → 进入离线模式，标记 loaded 避免重复请求
        error.value = '服务器无法连接，应用处于离线模式'
      } else {
        // 服务器可达但配置接口异常 → 使用默认配置继续
        error.value = '获取服务端配置失败，使用默认配置'
      }
      // 无论哪种情况都标记 loaded 防止卡住
      loaded.value = true
    } finally {
      loading.value = false
    }
  }

  /** 等待配置加载完成 */
  async function ensureLoaded(): Promise<void> {
    if (loaded.value) return
    await fetchConfig()
  }

  return {
    config,
    loaded,
    loading,
    error,
    fetchConfig,
    ensureLoaded,
  }
})
