import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { roomApi } from '@/api/room'
import { useAuthStore } from '@/stores/auth'
import type { Room, RoomMember, CreateRoomData, UpdateRoomData } from '@/types/room'

export const useRoomStore = defineStore('room', () => {
  // 状态
  const rooms = ref<Room[]>([])
  const currentRoom = ref<Room | null>(null)
  const members = ref<RoomMember[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const roomOrder = ref<string[]>(JSON.parse(localStorage.getItem('room-order') || '[]'))

  // 获取认证状态
  const getAuthStore = () => useAuthStore()

  // 计算属性
  const roomMap = computed(() => {
    const map = new Map<string, Room>()
    for (const room of rooms.value) {
      map.set(room.id, room)
    }
    return map
  })

  // 获取房间列表
  async function fetchRooms(params?: { search?: string }) {
    loading.value = true
    error.value = null
    try {
      const res = await roomApi.listRooms(params)
      if (res.success && res.data) {
        rooms.value = res.data
      }
    } catch (err) {
      error.value = '获取聊天室列表失败'
      console.error('[RoomStore] fetchRooms error:', err)
    } finally {
      loading.value = false
    }
  }

  // 获取我加入的房间
  async function fetchMyRooms() {
    // 检查用户是否已登录
    const authStore = getAuthStore()
    if (!authStore.isAuthenticated) {
      console.log('[RoomStore] fetchMyRooms: 用户未登录，跳过加载')
      return
    }
    loading.value = true
    error.value = null
    try {
      const res = await roomApi.getMyRooms()
      console.log('[RoomStore] fetchMyRooms: 获取到房间数据', res)
      if (res.success && Array.isArray(res.data)) {
        rooms.value = res.data
      } else {
        console.error('[RoomStore] fetchMyRooms: 返回数据格式异常', res)
        rooms.value = []
      }
    } catch (err) {
      error.value = '获取我的聊天室失败'
      console.error('[RoomStore] fetchMyRooms error:', err)
    } finally {
      loading.value = false
    }
  }

  // 获取房间详情
  async function fetchRoomDetail(roomId: string) {
    try {
      const res = await roomApi.getRoom(roomId)
      if (res.success && res.data) {
        currentRoom.value = res.data
      }
    } catch (err) {
      console.error('[RoomStore] fetchRoomDetail error:', err)
    }
  }

  // 获取房间成员
  async function fetchMembers(roomId: string) {
    try {
      const res = await roomApi.getRoomMembers(roomId)
      if (res.success && res.data) {
        members.value = res.data
      }
    } catch (err) {
      console.error('[RoomStore] fetchMembers error:', err)
    }
  }

  // 创建房间
  async function createRoom(data: CreateRoomData): Promise<Room | null> {
    error.value = null
    try {
      const res = await roomApi.createRoom(data)
      if (res.success && res.data) {
        rooms.value.unshift(res.data)
        return res.data
      }
      return null
    } catch (err) {
      error.value = '创建聊天室失败'
      console.error('[RoomStore] createRoom error:', err)
      return null
    }
  }

  // 加入房间
  async function joinRoom(roomId: string): Promise<boolean> {
    try {
      await roomApi.joinRoom(roomId)
      return true
    } catch (err) {
      error.value = '加入聊天室失败'
      console.error('[RoomStore] joinRoom error:', err)
      return false
    }
  }

  // 更新房间信息
  async function updateRoom(roomId: string, data: UpdateRoomData): Promise<boolean> {
    error.value = null
    try {
      const res = await roomApi.updateRoom(roomId, data)
      if (res.success) {
        const room = rooms.value.find((r) => r.id === roomId)
        if (room && res.data) {
          Object.assign(room, res.data)
        }
        if (currentRoom.value?.id === roomId && res.data) {
          Object.assign(currentRoom.value, res.data)
        }
        return true
      }
      return false
    } catch (err) {
      error.value = '更新房间失败'
      console.error('[RoomStore] updateRoom error:', err)
      return false
    }
  }

  // 删除房间
  async function deleteRoom(roomId: string): Promise<boolean> {
    error.value = null
    try {
      const res = await roomApi.deleteRoom(roomId)
      if (res.success) {
        rooms.value = rooms.value.filter((r) => r.id !== roomId)
        if (currentRoom.value?.id === roomId) {
          currentRoom.value = null
          members.value = []
        }
        return true
      }
      return false
    } catch (err) {
      error.value = '删除房间失败'
      console.error('[RoomStore] deleteRoom error:', err)
      return false
    }
  }

  // 踢出成员
  async function kickMember(roomId: string, userId: string): Promise<boolean> {
    error.value = null
    try {
      const res = await roomApi.kickMember(roomId, userId)
      if (res.success) {
        members.value = members.value.filter((m) => m.user_id !== userId)
        return true
      }
      return false
    } catch (err) {
      error.value = '踢出成员失败'
      console.error('[RoomStore] kickMember error:', err)
      return false
    }
  }

  // 设置成员角色
  async function setMemberRole(roomId: string, userId: string, role: 'admin' | 'member'): Promise<boolean> {
    error.value = null
    try {
      const res = await roomApi.setMemberRole(roomId, userId, role)
      if (res.success) {
        const member = members.value.find((m) => m.user_id === userId)
        if (member) member.role = role
        return true
      }
      return false
    } catch (err) {
      error.value = '设置成员角色失败'
      console.error('[RoomStore] setMemberRole error:', err)
      return false
    }
  }

  // 离开房间
  async function leaveRoom(roomId: string): Promise<boolean> {
    try {
      await roomApi.leaveRoom(roomId)
      rooms.value = rooms.value.filter((r) => r.id !== roomId)
      if (currentRoom.value?.id === roomId) {
        currentRoom.value = null
        members.value = []
      }
      return true
    } catch (err) {
      error.value = '离开聊天室失败'
      console.error('[RoomStore] leaveRoom error:', err)
      return false
    }
  }

  // 更新成员在线状态（由 WebSocket UserStatusChanged 事件触发）
  function updateMemberStatus(userId: string, status: RoomMember['user_status']) {
    const member = members.value.find((m) => m.user_id === userId)
    if (member) {
      member.user_status = status
    }
  }

  // 重置当前房间
  function clearCurrentRoom() {
    currentRoom.value = null
    members.value = []
  }

  /**
   * 更新房间最新消息预览（用于 WebSocket 实时更新）
   * @param roomId 房间 ID
   * @param message 消息预览数据
   * @param incrementUnread 是否增加未读计数（默认为 true）
   */
  function updateRoomLastMessage(
    roomId: string,
    message: {
      id: string
      content: string
      sender_name: string
      created_at: string
    },
    incrementUnread: boolean = true,
  ) {
    const room = rooms.value.find((r) => r.id === roomId)
    if (room) {
      room.last_message = message
      if (incrementUnread && currentRoom.value?.id !== roomId) {
        room.unread_count = (room.unread_count || 0) + 1
      }
    }
  }

  /**
   * 增加房间未读消息数
   * @param roomId 房间 ID
   */
  function incrementUnreadCount(roomId: string) {
    const room = rooms.value.find((r) => r.id === roomId)
    if (room) {
      room.unread_count = (room.unread_count || 0) + 1
    }
  }

  /**
   * 清除房间未读消息数
   * @param roomId 房间 ID
   */
  function clearUnreadCount(roomId: string) {
    const room = rooms.value.find((r) => r.id === roomId)
    if (room) {
      room.unread_count = 0
    }
  }

  function setRoomOrder(order: string[]) {
    roomOrder.value = order
    localStorage.setItem('room-order', JSON.stringify(order))
  }

  function $reset() {
    rooms.value = []
    currentRoom.value = null
    members.value = []
    loading.value = false
    error.value = null
  }

  return {
    rooms,
    currentRoom,
    members,
    loading,
    error,
    roomMap,
    fetchRooms,
    fetchMyRooms,
    fetchRoomDetail,
    fetchMembers,
    createRoom,
    joinRoom,
    updateMemberStatus,
    updateRoom,
    deleteRoom,
    kickMember,
    setMemberRole,
    leaveRoom,
	roomOrder,
    clearCurrentRoom,
    updateRoomLastMessage,
    incrementUnreadCount,
    clearUnreadCount,
    setRoomOrder,
    $reset,
  }
})
