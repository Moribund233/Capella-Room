import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { roomApi } from '@/api/room'
import type { Room, RoomMember, CreateRoomData } from '@/types/room'

export const useRoomStore = defineStore('room', () => {
  // 状态
  const rooms = ref<Room[]>([])
  const currentRoom = ref<Room | null>(null)
  const members = ref<RoomMember[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

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
    loading.value = true
    error.value = null
    try {
      const data = await roomApi.getMyRooms()
      rooms.value = data
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

  // 重置当前房间
  function clearCurrentRoom() {
    currentRoom.value = null
    members.value = []
  }

  /**
   * 更新房间最新消息预览（用于 WebSocket 实时更新）
   * @param roomId 房间 ID
   * @param message 消息预览数据
   */
  function updateRoomLastMessage(
    roomId: string,
    message: {
      id: string
      content: string
      sender_name: string
      created_at: string
    },
  ) {
    const room = rooms.value.find((r) => r.id === roomId)
    if (room) {
      room.last_message = message
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
    leaveRoom,
    clearCurrentRoom,
    updateRoomLastMessage,
    incrementUnreadCount,
    clearUnreadCount,
    $reset,
  }
})
