import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useRoomStore } from '@/stores/room'
import { useAuthStore } from '@/stores/auth'
import { useWebSocketStore } from '@/stores/websocket'
import type { CreateRoomData } from '@/types/room'
import { ROUTE_PATHS } from '@/constants'

export function useRoom() {
  const router = useRouter()
  const roomStore = useRoomStore()
  const authStore = useAuthStore()
  const wsStore = useWebSocketStore()
  const { rooms, currentRoom, members, loading, error } = storeToRefs(roomStore)

  async function loadRooms() {
    // 仅在用户已登录时获取房间列表
    if (!authStore.isAuthenticated) {
      return
    }
    await roomStore.fetchMyRooms()
  }

  async function loadRoomDetail(roomId: string) {
    await Promise.all([
      roomStore.fetchRoomDetail(roomId),
      roomStore.fetchMembers(roomId),
    ])
  }

  async function createRoom(data: CreateRoomData) {
    const room = await roomStore.createRoom(data)
    if (room) {
      if (wsStore.isConnected) {
        wsStore.send('JoinRoom', { room_id: room.id })
      }
      router.push(`/room/${room.id}`)
    }
    return room
  }

  async function joinRoom(roomId: string) {
    const ok = await roomStore.joinRoom(roomId)
    if (ok) {
      if (wsStore.isConnected) {
        wsStore.send('JoinRoom', { room_id: roomId })
      }
      await roomStore.fetchMyRooms()
      router.push(`/room/${roomId}`)
    }
    return ok
  }

  async function leaveRoom(roomId: string) {
    if (wsStore.isConnected) {
      wsStore.send('LeaveRoom', { room_id: roomId })
    }
    const ok = await roomStore.leaveRoom(roomId)
    if (ok) {
      router.push(ROUTE_PATHS.CHAT)
    }
    return ok
  }

  return {
    rooms,
    currentRoom,
    members,
    loading,
    error,
    loadRooms,
    loadRoomDetail,
    createRoom,
    joinRoom,
    leaveRoom,
  }
}
