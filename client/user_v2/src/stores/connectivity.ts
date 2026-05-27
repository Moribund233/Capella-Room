import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { checkServerHealth } from '@/services/health'
import type { ServerStatus } from '@/services/health'

export const useConnectivityStore = defineStore('connectivity', () => {
  const serverStatus = ref<ServerStatus>('reachable')
  const checked = ref(false)

  const isOffline = computed(() => serverStatus.value === 'unreachable')

  async function probeServer(): Promise<ServerStatus> {
    const status = await checkServerHealth()
    serverStatus.value = status
    checked.value = true
    return status
  }

  function markOffline() {
    serverStatus.value = 'unreachable'
    checked.value = true
  }

  function reset() {
    serverStatus.value = 'reachable'
    checked.value = false
  }

  return {
    serverStatus,
    checked,
    isOffline,
    probeServer,
    markOffline,
    reset,
  }
})
