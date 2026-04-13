<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { MessageSquare, Users, Shield, RefreshCw } from 'lucide-vue-next'
import { getRooms, joinRoom, getRoomMembers, kickMember, type Room, type RoomMember } from '@/api'
import { useWebSocketStore } from '@/stores/websocket'
import { useAuthStore } from '@/stores/auth'
import { useMultiUserAuthStore } from '@/stores/multiUserAuth'
import { authManager, type TestUser } from '@/utils/authUtils'
import ChatPanel from '@/components/room/ChatPanel.vue'
import RoomMembers from '@/components/room/RoomMembers.vue'
import OnlineUsers from '@/components/room/OnlineUsers.vue'
import ToolBar from '@/components/room/ToolBar.vue'
import type { RoomMemberWithTestUser } from '@/utils/room/types'

const message = useMessage()
const wsStore = useWebSocketStore()
const authStore = useAuthStore()
const multiUserAuthStore = useMultiUserAuthStore()

// ========== 状态 ==========
const rooms = ref<Room[]>([])
const selectedRoomId = ref<string>('')
const loading = ref(false)
const joiningRoom = ref(false)
const onlineUsers = ref<TestUser[]>([])
const currentTestUser = ref<TestUser | null>(null)
const roomMembers = ref<RoomMember[]>([])

// ========== 计算属性 ==========
const selectedRoom = computed(() => {
  return rooms.value.find(r => r.id === selectedRoomId.value)
})

const canJoinRoom = computed(() => {
  return selectedRoomId.value && wsStore.isConnected && !joiningRoom.value
})

/** 合并房间成员和测试用户信息的计算属性 */
const roomMembersWithTestUser = computed((): RoomMemberWithTestUser[] => {
  return roomMembers.value.map(member => {
    // 查找是否有对应的测试用户（通过 user_id 匹配）
    const testUser = multiUserAuthStore.testUsers.find(tu => tu.id === member.user_id)
    return {
      ...member,
      testUser: testUser || undefined,
    }
  })
})

// ========== 房间加载 ==========
const loadRooms = async () => {
  loading.value = true
  try {
    const data = await getRooms()
    rooms.value = data
    if (data.length > 0 && !selectedRoomId.value) {
      const firstRoom = data[0]
      if (firstRoom) {
        selectedRoomId.value = firstRoom.id
      }
    }
  } catch (error) {
    message.error('加载房间列表失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// ========== 加载房间成员 ==========
const loadRoomMembers = async () => {
  if (!selectedRoomId.value) {
    roomMembers.value = []
    return
  }

  try {
    const members = await getRoomMembers(selectedRoomId.value)
    roomMembers.value = members
  } catch (error) {
    console.error('加载房间成员失败:', error)
    roomMembers.value = []
  }
}

// ========== 加入房间 ==========
const handleJoinRoom = async () => {
  if (!selectedRoomId.value || !wsStore.isConnected) return

  const roomId = selectedRoomId.value
  const room = rooms.value.find(r => r.id === roomId)
  if (!room) return

  // 检查是否已加入
  if (wsStore.joinedRooms.includes(roomId)) {
    message.info('已加入该房间')
    return
  }

  joiningRoom.value = true

  try {
    // 获取当前用户ID
    const currentUserId = authStore.user?.id
    const isOwner = room.owner?.id === currentUserId

    // 只有非创建者才需要调用 HTTP API 加入房间
    if (!isOwner) {
      try {
        await joinRoom(roomId)
      } catch (error: any) {
        const errorMsg = error.message || ''
        // 409 冲突或已经是成员的错误可以忽略
        if (!errorMsg.includes('409') && !errorMsg.includes('already') && !errorMsg.includes('已经是')) {
          throw error
        }
      }
    }

    // 通过 WebSocket 加入房间
    wsStore.send({
      type: 'JoinRoom',
      payload: { room_id: roomId }
    })

    message.success(`已加入房间: ${room.name}`)
  } catch (error: any) {
    message.error(`加入房间失败: ${error.message || '未知错误'}`)
  } finally {
    joiningRoom.value = false
  }
}

// ========== 在线用户变更 ==========
const handleOnlineUsersChanged = (users: TestUser[]) => {
  onlineUsers.value = users
}

// ========== 选择测试用户 ==========
const handleSelectTestUser = (user: TestUser) => {
  currentTestUser.value = user
  message.info(`已选择测试用户: ${user.username}`)
}

// ========== 获取当前聊天用户 ==========
/**
 * 获取当前用于聊天的用户
 * 始终返回主认证用户（authStore.user），因为 ChatPanel 需要知道"谁在查看聊天"
 * 测试用户仅用于测试工具发送消息，不影响消息布局
 */
const getCurrentUserForChat = (): TestUser | null => {
  // 始终使用主认证用户
  if (authStore.user) {
    return {
      id: authStore.user.id,
      username: authStore.user.username,
      email: authStore.user.email,
      role: authStore.user.role,
      accessToken: '',
      refreshToken: '',
      isActive: true,
    }
  }
  return null
}

// ========== 测试用户加入房间 ==========
const handleTestUserJoinRoom = async (user: TestUser) => {
  if (!selectedRoomId.value) {
    message.warning('请先选择房间')
    return
  }

  const roomId = selectedRoomId.value
  const room = rooms.value.find(r => r.id === roomId)
  if (!room) {
    message.error('房间不存在')
    return
  }

  try {
    // 使用测试用户的 token 发送加入房间请求
    await authManager.requestAsUser(user.id, `/api/v1/rooms/${roomId}/join`, {
      method: 'POST',
    })

    message.success(`${user.username} 已加入房间: ${room.name}`)

    // 刷新成员列表
    setTimeout(() => {
      // 触发 RoomMembers 组件重新加载成员
      const event = new CustomEvent('refresh-room-members', { detail: { roomId } })
      window.dispatchEvent(event)
    }, 500)
  } catch (error: any) {
    const errorMsg = error.message || ''
    // 如果用户已经是成员，显示提示而不是错误
    if (errorMsg.includes('409') || errorMsg.includes('already') || errorMsg.includes('已经是')) {
      message.info(`${user.username} 已经是房间成员`)
    } else {
      message.error(`${user.username} 加入房间失败: ${errorMsg || '未知错误'}`)
    }
  }
}

// ========== 踢出成员 ==========
const handleKickMember = async (memberId: string) => {
  if (!selectedRoomId.value) {
    message.warning('请先选择房间')
    return
  }

  try {
    await kickMember(selectedRoomId.value, memberId)
    message.success('成员已被踢出')
    // 刷新成员列表
    await loadRoomMembers()
  } catch (error: any) {
    const errorMsg = error.message || '未知错误'
    message.error(`踢出成员失败: ${errorMsg}`)
  }
}

// ========== 设置管理员 ==========
const handleSetAdmin = (memberId: string) => {
  message.info('设置管理员功能需要后端 API 支持')
}

// ========== 工具栏发送消息 ==========
const handleToolSendMessage = async (userId: string, content: string) => {
  if (!selectedRoomId.value) {
    message.warning('请先选择房间')
    return
  }

  const user = onlineUsers.value.find(u => u.id === userId)
  if (!user) {
    message.error('未找到测试用户')
    return
  }

  try {
    // 使用测试用户的 token 发送消息
    await authManager.requestAsUser(userId, '/api/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        room_id: selectedRoomId.value,
        content: content,
      }),
    })
  } catch (error: any) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    throw new Error(`发送消息失败: ${errorMsg}`)
  }
}

// ========== 初始化 ==========
onMounted(() => {
  loadRooms()

  // 连接 WebSocket
  if (!wsStore.isConnected && !wsStore.isConnecting) {
    wsStore.connect()
  }
})

// 监听房间变化，自动加入并加载成员
watch(selectedRoomId, (newRoomId) => {
  if (newRoomId) {
    loadRoomMembers()
    if (wsStore.isConnected) {
      handleJoinRoom()
    }
  } else {
    roomMembers.value = []
  }
})

// 监听 WebSocket 连接状态
watch(() => wsStore.isConnected, (connected) => {
  if (connected && selectedRoomId.value) {
    handleJoinRoom()
  }
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <MessageSquare
          class="icon-lg"
          style="display: inline; vertical-align: middle; margin-right: 8px"
        />
        房间测试
      </h1>
      <p class="page-subtitle">测试房间聊天、成员管理和多用户场景</p>
    </div>

    <!-- 房间选择器 -->
    <n-card style="margin-bottom: var(--space-lg)">
      <n-space align="center" justify="space-between">
        <n-space>
          <n-select
            v-model:value="selectedRoomId"
            :options="rooms.map(r => ({ label: r.name, value: r.id }))"
            placeholder="选择房间"
            style="width: 240px"
            :loading="loading"
          />
          <n-button text @click="loadRooms">
            <template #icon>
              <RefreshCw class="icon-sm" />
            </template>
          </n-button>
        </n-space>

        <n-space align="center">
          <!-- 工具栏 -->
          <ToolBar
            :room-id="selectedRoomId"
            :room-members="roomMembersWithTestUser"
            @send-message="handleToolSendMessage"
          />
          <n-divider vertical />
          <n-tag v-if="selectedRoom" size="small">
            {{ selectedRoom.member_count || 0 }} 成员
          </n-tag>
          <n-tag :type="wsStore.isConnected ? 'success' : 'error'" size="small">
            {{ wsStore.isConnected ? '已连接' : '未连接' }}
          </n-tag>
          <n-button
            type="primary"
            :disabled="!canJoinRoom"
            :loading="joiningRoom"
            @click="handleJoinRoom"
          >
            <template #icon>
              <Shield class="icon-sm" />
            </template>
            加入房间
          </n-button>
        </n-space>
      </n-space>
    </n-card>

    <!-- 主布局 -->
    <div class="room-test-layout">
      <!-- 左侧：聊天界面 -->
      <div class="chat-section">
        <ChatPanel
          :room-id="selectedRoomId"
          :current-user="getCurrentUserForChat()"
          @reply="(msg) => console.log('回复消息:', msg)"
          @delete="(msgId) => console.log('删除消息:', msgId)"
        />
      </div>

      <!-- 右侧：成员和在线用户 -->
      <div class="sidebar-section">
        <RoomMembers
          :room-id="selectedRoomId"
          @kick="handleKickMember"
          @set-admin="handleSetAdmin"
        />
        <OnlineUsers
          :room-id="selectedRoomId"
          @users-changed="handleOnlineUsersChanged"
          @select-user="handleSelectTestUser"
          @join-room="handleTestUserJoinRoom"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.room-test-layout {
  display: flex;
  gap: var(--space-lg);
  height: calc(100vh - 280px);
  min-height: 500px;
}

.chat-section {
  flex: 1;
  min-width: 0;
}

.sidebar-section {
  flex: 0 0 320px;
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.sidebar-section > * {
  flex: 1;
  min-height: 0;
}

/* 移动端适配 */
@media screen and (max-width: 1024px) {
  .room-test-layout {
    flex-direction: column;
    height: auto;
  }

  .chat-section {
    height: 500px;
  }

  .sidebar-section {
    flex: 1 1 100%;
    flex-direction: row;
  }

  .sidebar-section > * {
    flex: 1;
  }
}

@media screen and (max-width: 767px) {
  .sidebar-section {
    flex-direction: column;
  }
}
</style>
