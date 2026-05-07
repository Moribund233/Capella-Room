<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NModal,
  NList,
  NListItem,
  NThing,
  NTag,
  NButton,
  NSpace,
  NPopconfirm,
  NSelect,
  NEmpty,
  NSkeleton,
  useMessage,
} from 'naive-ui'
import { Crown, Shield, User, UserX } from 'lucide-vue-next'
import { roomApi } from '@/api/room'
import { useAuthStore } from '@/stores/auth'
import type { RoomMember } from '@/types/room'
import type { Component } from 'vue'

/**
 * 组件属性定义
 */
interface Props {
  /** 房间ID */
  roomId: string
  /** 是否显示 */
  visible: boolean
  /** 成员列表 */
  members: RoomMember[]
  /** 加载状态 */
  loading?: boolean
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 关闭弹窗 */
  (e: 'update:visible', value: boolean): void
  /** 成员变更 */
  (e: 'membersChanged'): void
}

const emit = defineEmits<Emits>()

const message = useMessage()
const authStore = useAuthStore()

/** 操作加载状态 */
const actionLoading = ref<string | null>(null)

/**
 * 当前用户角色
 */
const currentUserRole = computed(() => {
  const currentMember = props.members.find(m => m.user_id === authStore.user?.id)
  return currentMember?.role || 'member'
})

/**
 * 是否为房间所有者
 */
const isOwner = computed(() => currentUserRole.value === 'owner')

/**
 * 是否为管理员（包括所有者）
 */
const isAdmin = computed(() => ['owner', 'admin'].includes(currentUserRole.value))

/**
 * 角色图标映射
 */
const roleIconMap: Record<string, Component> = {
  owner: Crown,
  admin: Shield,
  member: User,
}

/**
 * 角色标签映射
 */
const roleLabelMap: Record<string, string> = {
  owner: '房主',
  admin: '管理员',
  member: '成员',
}

/**
 * 角色标签类型映射
 */
const roleTagTypeMap: Record<string, 'success' | 'warning' | 'default'> = {
  owner: 'success',
  admin: 'warning',
  member: 'default',
}

/**
 * 角色选项（用于下拉选择）
 */
const roleOptions = [
  { label: '成员', value: 'member' },
  { label: '管理员', value: 'admin' },
]

/**
 * 判断是否可管理该成员
 */
function canManage(member: RoomMember): boolean {
  // 不能管理自己
  if (member.user_id === authStore.user?.id) return false
  // 不能管理房主
  if (member.role === 'owner') return false
  // 管理员不能管理其他管理员
  if (member.role === 'admin' && currentUserRole.value !== 'owner') return false
  return true
}

/**
 * 判断是否可修改角色
 */
function canChangeRole(member: RoomMember): boolean {
  return isOwner.value && canManage(member)
}

/**
 * 判断是否可踢出
 */
function canKick(member: RoomMember): boolean {
  return isAdmin.value && canManage(member)
}

/**
 * 踢出成员
 */
async function handleKick(member: RoomMember) {
  if (!canKick(member)) return

  actionLoading.value = member.user_id

  try {
    const res = await roomApi.kickMember(props.roomId, member.user_id)
    if (res.success) {
      message.success(`已将 ${member.username} 移出房间`)
      emit('membersChanged')
    } else {
      message.error('操作失败')
    }
  } catch (err) {
    message.error('移出成员失败')
    console.error('[RoomMemberManager] kick error:', err)
  } finally {
    actionLoading.value = null
  }
}

/**
 * 设置成员角色
 */
async function handleRoleChange(member: RoomMember, newRole: string) {
  if (!canChangeRole(member)) return

  actionLoading.value = member.user_id

  try {
    const res = await roomApi.setMemberRole(props.roomId, member.user_id, newRole as 'admin' | 'member')
    if (res.success) {
      message.success(`已将 ${member.username} 设置为${roleLabelMap[newRole]}`)
      emit('membersChanged')
    } else {
      message.error('操作失败')
    }
  } catch (err) {
    message.error('设置角色失败')
    console.error('[RoomMemberManager] set role error:', err)
  } finally {
    actionLoading.value = null
  }
}

/**
 * 获取在线状态标签
 */
function getStatusLabel(status: string): string {
  const map: Record<string, string> = {
    online: '在线',
    away: '离开',
    busy: '忙碌',
    offline: '离线',
  }
  return map[status] || '离线'
}

/**
 * 获取在线状态类型
 */
function getStatusType(status: string): 'success' | 'warning' | 'error' | 'default' {
  const map: Record<string, 'success' | 'warning' | 'error' | 'default'> = {
    online: 'success',
    away: 'warning',
    busy: 'error',
    offline: 'default',
  }
  return map[status] || 'default'
}

/**
 * 处理关闭
 */
function handleClose() {
  emit('update:visible', false)
}
</script>

<template>
  <NModal
    :show="visible"
    preset="card"
    title="成员管理"
    style="width: 90%; max-width: 600px; max-height: 80vh"
    @close="handleClose"
    @mask-click="handleClose"
  >
    <div class="room-member-manager">
      <!-- 加载中 -->
      <div v-if="loading" class="room-member-manager__loading">
        <NSkeleton text :repeat="5" />
      </div>

      <!-- 空状态 -->
      <NEmpty
        v-else-if="members.length === 0"
        description="暂无成员"
      />

      <!-- 成员列表 -->
      <NList v-else bordered>
        <NListItem
          v-for="member in members"
          :key="member.user_id"
        >
          <NThing>
            <template #avatar>
              <div class="room-member-manager__avatar">
                {{ member.username.charAt(0).toUpperCase() }}
              </div>
            </template>

            <template #header>
              <div class="room-member-manager__header">
                <span class="room-member-manager__username">
                  {{ member.username }}
                  <span
                    v-if="member.user_id === authStore.user?.id"
                    class="room-member-manager__self-tag"
                  >
                    (我)
                  </span>
                </span>
                <div class="room-member-manager__tags">
                  <NTag
                    :type="roleTagTypeMap[member.role]"
                    size="small"
                  >
                    <template #icon>
                      <component :is="roleIconMap[member.role]" :size="12" />
                    </template>
                    {{ roleLabelMap[member.role] }}
                  </NTag>
                  <NTag
                    :type="getStatusType(member.user_status)"
                    size="small"
                  >
                    {{ getStatusLabel(member.user_status) }}
                  </NTag>
                </div>
              </div>
            </template>

            <template #description>
              <span class="room-member-manager__email">{{ member.email }}</span>
            </template>

            <template #action>
              <NSpace v-if="canManage(member)">
                <!-- 角色选择（仅房主可用） -->
                <NSelect
                  v-if="canChangeRole(member)"
                  :value="member.role"
                  :options="roleOptions"
                  size="small"
                  style="width: 100px"
                  :disabled="actionLoading === member.user_id"
                  @update:value="(val) => handleRoleChange(member, val)"
                />

                <!-- 踢出按钮 -->
                <NPopconfirm
                  v-if="canKick(member)"
                  @positive-click="handleKick(member)"
                >
                  <template #trigger>
                    <NButton
                      size="small"
                      type="error"
                      ghost
                      :loading="actionLoading === member.user_id"
                    >
                      <template #icon>
                        <UserX :size="14" />
                      </template>
                      移出
                    </NButton>
                  </template>
                  <span>确定要将 {{ member.username }} 移出房间吗？</span>
                </NPopconfirm>
              </NSpace>
            </template>
          </NThing>
        </NListItem>
      </NList>
    </div>
  </NModal>
</template>

<style scoped>
.room-member-manager {
  max-height: 60vh;
  overflow-y: auto;
}

.room-member-manager__loading {
  padding: 20px;
}

.room-member-manager__avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--color-primary);
  color: var(--color-white);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 600;
}

.room-member-manager__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.room-member-manager__username {
  font-weight: 500;
  color: var(--color-text-primary);
}

.room-member-manager__self-tag {
  color: var(--color-text-secondary);
  font-weight: normal;
}

.room-member-manager__tags {
  display: flex;
  gap: 6px;
}

.room-member-manager__email {
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-secondary);
}
</style>
