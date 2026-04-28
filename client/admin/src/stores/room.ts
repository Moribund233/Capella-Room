import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { roomsApi, adminApi } from '@/api'
import type { RoomInfo, MemberInfo } from '@/api/rooms'
import type { AdminMessageInfo } from '@/api/admin'

/**
 * 房间管理 Store
 * 用于管理房间列表、成员、消息等状态
 */
export const useRoomStore = defineStore('room', () => {
  // ==================== 状态 ====================

  /** 房间列表数据 */
  const rooms = ref<RoomInfo[]>([])
  /** 加载状态 */
  const loading = ref(false)
  /** 总房间数 */
  const total = ref(0)
  /** 当前页码 */
  const page = ref(1)
  /** 每页数量 */
  const pageSize = ref(10)
  /** 选中的房间 keys */
  const selectedKeys = ref<(string | number)[]>([])
  /** 当前选中的房间 */
  const currentRoom = ref<RoomInfo | null>(null)

  /** 搜索参数 */
  const searchParams = ref({
    keyword: '',
  })

  /** 当前搜索参数缓存（用于刷新） */
  const currentSearchParams = ref({
    keyword: '',
  })

  // ==================== 成员管理状态 ====================

  /** 成员列表 */
  const members = ref<MemberInfo[]>([])
  /** 成员列表加载状态 */
  const membersLoading = ref(false)

  // ==================== 消息管理状态 ====================

  /** 消息列表 */
  const messages = ref<AdminMessageInfo[]>([])
  /** 消息总数 */
  const messagesTotal = ref(0)
  /** 消息页码 */
  const messagesPage = ref(1)
  /** 消息每页数量 */
  const messagesPageSize = ref(50)
  /** 消息加载状态 */
  const messagesLoading = ref(false)

  // ==================== Getters ====================

  /** 是否有选中的房间 */
  const hasSelectedRoom = computed(() => !!currentRoom.value)

  /** 当前房间ID */
  const currentRoomId = computed(() => currentRoom.value?.id || '')

  // ==================== Actions ====================

  /**
   * 获取房间列表
   * @param params 搜索参数
   */
  async function fetchRoomList(params: {
    keyword?: string
    page?: number
    pageSize?: number
  } = {}) {
    loading.value = true

    try {
      const response = await roomsApi.getRoomList({
        search: params.keyword || undefined,
        limit: params.pageSize ?? pageSize.value,
        offset: ((params.page ?? page.value) - 1) * (params.pageSize ?? pageSize.value),
      })

      if (response.success && response.data) {
        rooms.value = response.data
        // 注意：API 返回的是数组，total 需要通过其他方式获取
        // 这里暂时使用数组长度，实际应该通过 API 返回的总数
        total.value = response.data.length
        page.value = params.page ?? page.value
        pageSize.value = params.pageSize ?? pageSize.value
        return true
      }
      return false
    } catch (error) {
      console.error('获取房间列表失败:', error)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 刷新当前列表
   */
  async function refreshRooms() {
    return fetchRoomList({
      keyword: currentSearchParams.value.keyword,
      page: page.value,
      pageSize: pageSize.value,
    })
  }

  /**
   * 获取房间详情
   * @param roomId 房间ID
   */
  async function fetchRoomDetail(roomId: string): Promise<RoomInfo | null> {
    try {
      const response = await roomsApi.getRoomDetail(roomId)
      if (response.success && response.data) {
        currentRoom.value = response.data
        return response.data
      }
      return null
    } catch (error) {
      console.error('获取房间详情失败:', error)
      return null
    }
  }

  /**
   * 获取成员列表
   * @param roomId 房间ID
   */
  async function fetchMembers(roomId: string) {
    membersLoading.value = true
    try {
      const response = await roomsApi.getMembers(roomId)
      if (response.success && response.data) {
        members.value = response.data
        return true
      }
      return false
    } catch (error) {
      console.error('获取成员列表失败:', error)
      return false
    } finally {
      membersLoading.value = false
    }
  }

  /**
   * 踢出成员（使用管理员接口）
   * @param roomId 房间ID
   * @param userId 用户ID
   */
  async function kickMember(roomId: string, userId: string) {
    try {
      const response = await adminApi.kickRoomMember(roomId, userId)
      if (response.success) {
        // 从列表中移除该成员
        members.value = members.value.filter(m => m.user_id !== userId)
        return true
      }
      return false
    } catch (error) {
      console.error('踢出成员失败:', error)
      return false
    }
  }

  /**
   * 设置成员角色（使用管理员接口）
   * @param roomId 房间ID
   * @param userId 用户ID
   * @param role 角色
   */
  async function setMemberRole(roomId: string, userId: string, role: 'admin' | 'member' | 'owner') {
    try {
      const response = await adminApi.setRoomMemberRole(roomId, userId, role)
      if (response.success) {
        // 更新本地成员角色
        const member = members.value.find(m => m.user_id === userId)
        if (member) {
          member.role = role
        }
        return true
      }
      return false
    } catch (error) {
      console.error('设置成员角色失败:', error)
      return false
    }
  }

  /**
   * 获取房间消息
   * @param roomId 房间ID
   * @param params 分页参数
   */
  async function fetchRoomMessages(
    roomId: string,
    params: {
      page?: number
      pageSize?: number
      search?: string
    } = {}
  ) {
    messagesLoading.value = true
    try {
      const response = await adminApi.getRoomMessages(roomId, {
        page: params.page ?? messagesPage.value,
        page_size: params.pageSize ?? messagesPageSize.value,
        search: params.search,
      })
      if (response.success && response.data) {
        // GET /admin/rooms/:room_id/messages 返回数组格式
        messages.value = response.data
        messagesTotal.value = response.data.length
        messagesPage.value = params.page ?? messagesPage.value
        messagesPageSize.value = params.pageSize ?? messagesPageSize.value
        return true
      }
      return false
    } catch (error) {
      console.error('获取消息列表失败:', error)
      return false
    } finally {
      messagesLoading.value = false
    }
  }

  /**
   * 删除房间
   * @param roomId 房间ID
   */
  async function deleteRoom(roomId: string) {
    try {
      const response = await roomsApi.deleteRoom(roomId)
      if (response.success) {
        await refreshRooms()
        return true
      }
      return false
    } catch (error) {
      console.error('删除房间失败:', error)
      return false
    }
  }

  /**
   * 删除消息
   * @param messageId 消息ID
   */
  async function deleteMessage(messageId: string) {
    try {
      const response = await adminApi.deleteMessage(messageId)
      if (response.success) {
        // 刷新当前消息列表
        if (currentRoom.value) {
          await fetchRoomMessages(currentRoom.value.id)
        }
        return true
      }
      return false
    } catch (error) {
      console.error('删除消息失败:', error)
      return false
    }
  }

  /**
   * 选择房间
   * @param room 房间信息
   */
  function selectRoom(room: RoomInfo) {
    currentRoom.value = room
    selectedKeys.value = [room.id]
  }

  /**
   * 搜索房间
   * @param keyword 关键词
   */
  async function searchRooms(keyword: string) {
    searchParams.value.keyword = keyword
    currentSearchParams.value.keyword = keyword
    selectedKeys.value = []
    currentRoom.value = null

    return fetchRoomList({
      keyword,
      page: 1,
      pageSize: pageSize.value,
    })
  }

  /**
   * 重置搜索
   */
  async function resetSearch() {
    searchParams.value.keyword = ''
    currentSearchParams.value.keyword = ''
    selectedKeys.value = []
    currentRoom.value = null
    page.value = 1

    return fetchRoomList({ page: 1, pageSize: pageSize.value })
  }

  /**
   * 分页变化
   * @param newPage 新页码
   * @param newPageSize 新每页数量
   */
  async function handlePageChange(newPage: number, newPageSize: number) {
    selectedKeys.value = []

    return fetchRoomList({
      keyword: currentSearchParams.value.keyword,
      page: newPage,
      pageSize: newPageSize,
    })
  }

  /**
   * 清空当前房间
   */
  function clearCurrentRoom() {
    currentRoom.value = null
    selectedKeys.value = []
  }

  /**
   * 重置所有状态
   */
  function $reset() {
    rooms.value = []
    loading.value = false
    total.value = 0
    page.value = 1
    pageSize.value = 10
    selectedKeys.value = []
    currentRoom.value = null
    searchParams.value = { keyword: '' }
    currentSearchParams.value = { keyword: '' }
    members.value = []
    membersLoading.value = false
    messages.value = []
    messagesTotal.value = 0
    messagesPage.value = 1
    messagesPageSize.value = 50
    messagesLoading.value = false
  }

  return {
    // 状态
    rooms,
    loading,
    total,
    page,
    pageSize,
    selectedKeys,
    currentRoom,
    searchParams,
    members,
    membersLoading,
    messages,
    messagesTotal,
    messagesPage,
    messagesPageSize,
    messagesLoading,
    // Getters
    hasSelectedRoom,
    currentRoomId,
    // Actions
    fetchRoomList,
    refreshRooms,
    fetchRoomDetail,
    fetchMembers,
    kickMember,
    setMemberRole,
    fetchRoomMessages,
    deleteRoom,
    deleteMessage,
    selectRoom,
    searchRooms,
    resetSearch,
    handlePageChange,
    clearCurrentRoom,
    $reset,
  }
})
