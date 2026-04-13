<script setup lang="ts">
import { ref, watch, computed, h, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import { Crown, Shield, User, UserX, LogIn } from 'lucide-vue-next'
import { getRoomMembers, type RoomMember } from '@/api'
import { useMultiUserWebSocketStore } from '@/stores/multiUserWebSocket'
import { useMultiUserAuthStore } from '@/stores/multiUserAuth'

const props = defineProps<{
  roomId: string
}>()

const emit = defineEmits<{
  (e: 'kick', memberId: string): void
  (e: 'setAdmin', memberId: string): void
}>()

const message = useMessage()
const wsStore = useMultiUserWebSocketStore()
const authStore = useMultiUserAuthStore()

// ========== 状态 ==========
const members = ref<RoomMember[]>([])
const loading = ref(false)

// ========== 计算属性 ==========
const sortedMembers = computed(() => {
  const roleOrder = { owner: 0, admin: 1, member: 2 }
  return [...(members.value || [])].sort((a, b) => {
    return roleOrder[a.role] - roleOrder[b.role]
  })
})

/**
 * 检查成员是否在线（已加入当前房间）
 * @param userId 用户ID
 */
const isMemberOnline = (userId: string): boolean => {
  const isOnline = wsStore.isUserInRoom(userId, props.roomId)
  return isOnline
}

const ownerCount = computed(() => (members.value || []).filter(m => m.role === 'owner').length)
const adminCount = computed(() => (members.value || []).filter(m => m.role === 'admin').length)
const memberCount = computed(() => (members.value || []).filter(m => m.role === 'member').length)

// ========== 加载成员 ==========
const loadMembers = async () => {
  if (!props.roomId) {
    console.log('[RoomMembers] 无房间ID，跳过加载')
    return
  }

  loading.value = true
  console.log('[RoomMembers] 开始加载成员，房间ID:', props.roomId)

  try {
    const membersData = await getRoomMembers(props.roomId)
    console.log('[RoomMembers] 加载成功，成员数:', membersData.length)
    console.log('[RoomMembers] 成员数据:', membersData)
    members.value = membersData
  } catch (error) {
    message.error('加载房间成员失败')
    console.error('[RoomMembers] 加载成员错误:', error)
  } finally {
    loading.value = false
  }
}

// ========== 获取角色图标 ==========
const getRoleIcon = (role: string) => {
  switch (role) {
    case 'owner':
      return Crown
    case 'admin':
      return Shield
    default:
      return User
  }
}

// ========== 获取角色颜色 ==========
const getRoleColor = (role: string): string => {
  switch (role) {
    case 'owner':
      return '#f0a020'
    case 'admin':
      return '#2080f0'
    default:
      return '#8c8c8c'
  }
}

// ========== 获取角色文本 ==========
const getRoleText = (role: string): string => {
  switch (role) {
    case 'owner':
      return '房主'
    case 'admin':
      return '管理员'
    default:
      return '成员'
  }
}

// ========== 踢出成员 ==========
const handleKick = (member: RoomMember) => {
  emit('kick', member.user_id)
}

// ========== 设置管理员 ==========
const handleSetAdmin = (member: RoomMember) => {
  emit('setAdmin', member.user_id)
}

// ========== 检查成员是否是测试用户 ==========
const isTestUser = (member: RoomMember): boolean => {
  return authStore.testUsers.some(u => u.id === member.user_id)
}

// ========== 获取测试用户信息 ==========
const getTestUser = (member: RoomMember) => {
  return authStore.testUsers.find(u => u.id === member.user_id)
}

// ========== 让测试用户加入房间 ==========
const handleJoinRoom = async (member: RoomMember) => {
  const testUser = getTestUser(member)
  if (!testUser) {
    message.error('未找到测试用户信息')
    return
  }

  // 检查是否已连接
  if (!wsStore.isUserConnected(testUser.id)) {
    message.error('测试用户未连接，请先完成认证')
    return
  }

  // 发送加入房间消息
  const success = wsStore.joinRoom(testUser.id, props.roomId)
  if (success) {
    message.success(`测试用户 ${member.username} 正在加入房间...`)
    // 等待服务器确认
    setTimeout(() => {
      if (wsStore.isUserInRoom(testUser.id, props.roomId)) {
        message.success(`测试用户 ${member.username} 已加入房间`)
      } else {
        message.error(`测试用户 ${member.username} 加入房间失败`)
      }
    }, 500)
  } else {
    message.error('发送加入房间消息失败')
  }
}

// 监听房间变化
watch(() => props.roomId, () => {
  members.value = []
  if (props.roomId) {
    loadMembers()
  }
}, { immediate: true })

// 组件挂载时初始化
onMounted(() => {
  // 从 sessionStorage 恢复测试用户数据
  authStore.initialize()
})
</script>

<template>
  <div class="room-members">
    <n-card title="房间成员" size="small">
      <template #header-extra>
        <n-space>
          <n-tag size="small" type="warning">{{ ownerCount }} 房主</n-tag>
          <n-tag size="small" type="info">{{ adminCount }} 管理员</n-tag>
          <n-tag size="small">{{ memberCount }} 成员</n-tag>
        </n-space>
      </template>

      <n-spin :show="loading">
        <n-list v-if="sortedMembers.length > 0" size="small">
          <n-list-item v-for="member in sortedMembers" :key="member.user_id">
            <n-thing>
              <template #avatar>
                <div class="avatar-wrapper">
                  <n-avatar
                    size="small"
                    :style="{ backgroundColor: getRoleColor(member.role) }"
                  >
                    {{ member.username.charAt(0).toUpperCase() }}
                  </n-avatar>
                  <span
                    v-if="isMemberOnline(member.user_id)"
                    class="online-indicator"
                    title="在线"
                  />
                </div>
              </template>
              <template #header>
                <n-space align="center" size="small">
                  <span>{{ member.username }}</span>
                  <n-icon
                    :component="getRoleIcon(member.role)"
                    :color="getRoleColor(member.role)"
                    size="14"
                  />
                  <n-tag :type="member.role === 'owner' ? 'warning' : member.role === 'admin' ? 'info' : 'default'" size="tiny">
                    {{ getRoleText(member.role) }}
                  </n-tag>
                </n-space>
              </template>
              <template #description>
                <span class="join-time">加入于 {{ new Date(member.joined_at).toLocaleDateString() }}</span>
              </template>
              <template #header-extra>
                <n-space size="small">
                  <!-- 测试用户加入房间按钮 -->
                  <n-button
                    v-if="isTestUser(member) && !isMemberOnline(member.user_id)"
                    size="tiny"
                    type="primary"
                    ghost
                    @click="handleJoinRoom(member)"
                  >
                    <template #icon>
                      <n-icon><LogIn :size="14" /></n-icon>
                    </template>
                    加入
                  </n-button>
                  <n-dropdown
                    v-if="member.role !== 'owner'"
                    :options="[
                      { label: '设为管理员', key: 'setAdmin', icon: () => h(Shield, { size: 14 }) },
                      { label: '踢出房间', key: 'kick', icon: () => h(UserX, { size: 14 }) },
                    ]"
                    @select="(key: string) => key === 'kick' ? handleKick(member) : handleSetAdmin(member)"
                  >
                    <n-button size="tiny" text>
                      <template #icon>
                        <n-icon><Shield class="icon-xs" /></n-icon>
                      </template>
                    </n-button>
                  </n-dropdown>
                </n-space>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>

        <n-empty v-else description="暂无成员" />
      </n-spin>
    </n-card>
  </div>
</template>

<style scoped>
.room-members {
  height: 100%;
  overflow-y: auto;
}

.join-time {
  font-size: 11px;
  color: var(--text-muted);
}

.icon-xs {
  width: 14px;
  height: 14px;
}

.avatar-wrapper {
  position: relative;
  display: inline-block;
}

.online-indicator {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 8px;
  height: 8px;
  background-color: #52c41a;
  border: 2px solid #fff;
  border-radius: 50%;
}
</style>
