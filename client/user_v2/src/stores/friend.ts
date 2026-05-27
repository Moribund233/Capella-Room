import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { friendApi } from '@/api/friend'
import { useAuthStore } from '@/stores/auth'
import type {
  Friend,
  FriendRequest,
  SendFriendRequestData,
} from '@/types/friend'

export const useFriendStore = defineStore('friend', () => {
  // 状态
  const friends = ref<Friend[]>([])
  const receivedRequests = ref<FriendRequest[]>([])
  const sentRequests = ref<FriendRequest[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 未读请求数（外部通过 WebSocket 递增）
  const unreadRequestCount = ref(0)

  const getAuthStore = () => useAuthStore()

  // 计算属性
  const onlineFriends = computed(() =>
    friends.value.filter((f) => f.friend.status === 'online'),
  )

  const pendingReceivedCount = computed(() =>
    receivedRequests.value.filter((r) => r.status === 'pending').length,
  )

  // 获取好友列表
  async function fetchFriends() {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) return

    loading.value = true
    error.value = null
    try {
      const res = await friendApi.getFriends()
      if (res.success && res.data) {
        friends.value = res.data.friends
      }
    } catch (err) {
      error.value = '获取好友列表失败'
      console.error('[FriendStore] fetchFriends error:', err)
    } finally {
      loading.value = false
    }
  }

  // 获取收到的请求
  async function fetchReceivedRequests() {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) return

    loading.value = true
    error.value = null
    try {
      const res = await friendApi.getReceivedRequests()
      if (res.success && res.data) {
        receivedRequests.value = res.data.requests
      }
    } catch (err) {
      error.value = '获取好友请求失败'
      console.error('[FriendStore] fetchReceivedRequests error:', err)
    } finally {
      loading.value = false
    }
  }

  // 获取发送的请求
  async function fetchSentRequests() {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) return

    loading.value = true
    error.value = null
    try {
      const res = await friendApi.getSentRequests()
      if (res.success && res.data) {
        sentRequests.value = res.data.requests
      }
    } catch (err) {
      error.value = '获取已发送请求失败'
      console.error('[FriendStore] fetchSentRequests error:', err)
    } finally {
      loading.value = false
    }
  }

  // 发送好友请求
  async function sendFriendRequest(data: SendFriendRequestData): Promise<boolean> {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) return false

    error.value = null
    try {
      const res = await friendApi.sendFriendRequest(data)
      if (res.success && res.data) {
        sentRequests.value.unshift(res.data)
        return true
      }
      error.value = res.message || '发送好友请求失败'
      return false
    } catch (err) {
      error.value = '发送好友请求失败'
      console.error('[FriendStore] sendFriendRequest error:', err)
      return false
    }
  }

  // 处理好友请求
  async function handleFriendRequest(requestId: string, accept: boolean): Promise<boolean> {
    error.value = null
    try {
      const res = await friendApi.handleFriendRequest(requestId, accept)
      if (res.success) {
        // 更新列表中该请求的状态
        const idx = receivedRequests.value.findIndex((r) => r.id === requestId)
        if (idx !== -1 && res.data) {
          receivedRequests.value[idx] = res.data
        }
        // 如果接受，刷新好友列表
        if (accept) {
          await fetchFriends()
        }
        return true
      }
      error.value = res.message || '处理请求失败'
      return false
    } catch (err) {
      error.value = '处理请求失败'
      console.error('[FriendStore] handleFriendRequest error:', err)
      return false
    }
  }

  // 取消好友请求
  async function cancelFriendRequest(requestId: string): Promise<boolean> {
    error.value = null
    try {
      const res = await friendApi.cancelFriendRequest(requestId)
      if (res.success) {
        sentRequests.value = sentRequests.value.filter((r) => r.id !== requestId)
        return true
      }
      error.value = res.message || '取消请求失败'
      return false
    } catch (err) {
      error.value = '取消请求失败'
      console.error('[FriendStore] cancelFriendRequest error:', err)
      return false
    }
  }

  // 删除好友
  async function deleteFriend(friendId: string): Promise<boolean> {
    error.value = null
    try {
      const res = await friendApi.deleteFriend(friendId)
      if (res.success) {
        friends.value = friends.value.filter((f) => f.id !== friendId)
        return true
      }
      error.value = res.message || '删除好友失败'
      return false
    } catch (err) {
      error.value = '删除好友失败'
      console.error('[FriendStore] deleteFriend error:', err)
      return false
    }
  }

  // 增加未读请求计数
  function incrementUnreadRequestCount() {
    unreadRequestCount.value++
  }

  // 清空未读请求计数
  function clearUnreadRequestCount() {
    unreadRequestCount.value = 0
  }

  function $reset() {
    friends.value = []
    receivedRequests.value = []
    sentRequests.value = []
    loading.value = false
    error.value = null
    unreadRequestCount.value = 0
  }

  return {
    friends,
    receivedRequests,
    sentRequests,
    loading,
    error,
    unreadRequestCount,
    onlineFriends,
    pendingReceivedCount,
    fetchFriends,
    fetchReceivedRequests,
    fetchSentRequests,
    sendFriendRequest,
    handleFriendRequest,
    cancelFriendRequest,
    deleteFriend,
    incrementUnreadRequestCount,
    clearUnreadRequestCount,
    $reset,
  }
})
