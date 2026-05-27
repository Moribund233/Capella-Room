import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { directRoomApi } from '@/api/directRoom'
import { useAuthStore } from '@/stores/auth'
import type { DirectRoom } from '@/types/room'

/**
 * 私聊房间 Store
 */
export const useDirectRoomStore = defineStore('directRoom', () => {
  // 状态
  const directRooms = ref<DirectRoom[]>([])
  const currentDirectRoom = ref<DirectRoom | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 获取认证状态
  const getAuthStore = () => useAuthStore()

  // 计算属性
  const directRoomMap = computed(() => {
    const map = new Map<string, DirectRoom>()
    for (const room of directRooms.value) {
      map.set(room.id, room)
    }
    return map
  })

  /**
   * 获取私聊房间列表
   */
  async function fetchDirectRooms() {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) {
      return
    }

    loading.value = true
    error.value = null
    try {
      const res = await directRoomApi.getDirectRooms()
      if (res.success && res.data) {
        directRooms.value = res.data
      }
    } catch (err) {
      error.value = '获取私聊列表失败'
      console.error('[DirectRoomStore] fetchDirectRooms error:', err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 创建或获取私聊房间
   * @param targetUserId - 对方用户ID
   * @returns 私聊房间信息
   */
  async function createOrGetDirectRoom(targetUserId: string): Promise<DirectRoom | null> {
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) {
      return null
    }

    // 检查是否已存在与该用户的私聊
    const existingRoom = directRooms.value.find(
      room => room.target_user.id === targetUserId
    )
    if (existingRoom) {
      currentDirectRoom.value = existingRoom
      return existingRoom
    }

    loading.value = true
    error.value = null
    try {
      const res = await directRoomApi.createDirectRoom(targetUserId)
      if (res.success && res.data) {
        // 如果是新创建的，添加到列表
        if (!directRoomMap.value.has(res.data.id)) {
          directRooms.value.unshift(res.data)
        }
        currentDirectRoom.value = res.data
        return res.data
      }
      return null
    } catch (err) {
      error.value = '创建私聊失败'
      console.error('[DirectRoomStore] createOrGetDirectRoom error:', err)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置当前私聊房间
   * @param room - 私聊房间
   */
  function setCurrentDirectRoom(room: DirectRoom | null) {
    currentDirectRoom.value = room
  }

  /**
   * 根据ID获取私聊房间
   * @param roomId - 房间ID
   */
  function getDirectRoomById(roomId: string): DirectRoom | undefined {
    return directRoomMap.value.get(roomId)
  }

  /**
   * 根据目标用户ID获取私聊房间
   * @param userId - 用户ID
   */
  function getDirectRoomByUserId(userId: string): DirectRoom | undefined {
    return directRooms.value.find(room => room.target_user.id === userId)
  }

  /**
   * 更新私聊房间的最后消息
   * @param roomId - 房间ID
   * @param content - 消息内容
   * @param senderName - 发送者名称
   */
  function updateLastMessage(roomId: string, content: string, senderName: string) {
    const room = directRoomMap.value.get(roomId)
    if (room) {
      room.last_message = {
        id: Date.now().toString(),
        content,
        sender_name: senderName,
        created_at: new Date().toISOString(),
      }
      // 移动到列表顶部
      const index = directRooms.value.findIndex(r => r.id === roomId)
      if (index > 0) {
        const removed = directRooms.value.splice(index, 1)[0]
        if (removed) {
          directRooms.value.unshift(removed)
        }
      }
    }
  }

  /**
   * 增加未读消息数
   * @param roomId - 房间ID
   */
  function incrementUnreadCount(roomId: string) {
    const room = directRoomMap.value.get(roomId)
    if (room) {
      room.unread_count = (room.unread_count || 0) + 1
    }
  }

  /**
   * 清除未读消息数
   * @param roomId - 房间ID
   */
  function clearUnreadCount(roomId: string) {
    const room = directRoomMap.value.get(roomId)
    if (room) {
      room.unread_count = 0
    }
  }

  /**
   * 重置状态
   */
  function reset() {
    directRooms.value = []
    currentDirectRoom.value = null
    loading.value = false
    error.value = null
  }

  return {
    // 状态
    directRooms,
    currentDirectRoom,
    loading,
    error,
    // 计算属性
    directRoomMap,
    // 方法
    fetchDirectRooms,
    createOrGetDirectRoom,
    setCurrentDirectRoom,
    getDirectRoomById,
    getDirectRoomByUserId,
    updateLastMessage,
    incrementUnreadCount,
    clearUnreadCount,
    reset,
  }
})
