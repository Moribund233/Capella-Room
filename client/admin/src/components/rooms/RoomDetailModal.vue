<script setup lang="ts">

import { NDescriptions, NDescriptionsItem, NTag, NSpace, NButton } from 'naive-ui'
import { MessageSquare, BarChart3, Edit } from 'lucide-vue-next'
import type { RoomInfo } from '@/api/rooms'

/**
 * 组件属性
 */
interface Props {
  /** 房间信息 */
  room: RoomInfo
}

const props = defineProps<Props>()

/**
 * 组件事件
 */
interface Emits {
  /** 查看消息 */
  (e: 'viewMessages', room: RoomInfo): void
  /** 查看分析 */
  (e: 'viewAnalytics', room: RoomInfo): void
  /** 编辑 */
  (e: 'edit', room: RoomInfo): void
}

const emit = defineEmits<Emits>()

/**
 * 格式化时间
 */
const formatTime = (time: string) => {
  return new Date(time).toLocaleString('zh-CN')
}

/**
 * 处理查看消息
 */
const handleViewMessages = () => {
  emit('viewMessages', props.room)
}

/**
 * 处理查看分析
 */
const handleViewAnalytics = () => {
  emit('viewAnalytics', props.room)
}

/**
 * 处理编辑
 */
const handleEdit = () => {
  emit('edit', props.room)
}
</script>

<template>
  <div class="room-detail-modal">
    <NDescriptions bordered :column="2">
      <NDescriptionsItem label="房间ID">
        {{ room.id }}
      </NDescriptionsItem>
      <NDescriptionsItem label="房间名称">
        {{ room.name }}
      </NDescriptionsItem>
      <NDescriptionsItem label="房间类型" :span="2">
        <NTag :type="room.is_private ? 'error' : 'success'">
          {{ room.is_private ? '私有' : '公开' }}
        </NTag>
      </NDescriptionsItem>
      <NDescriptionsItem label="房间描述" :span="2">
        {{ room.description || '暂无描述' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="房主">
        {{ room.owner?.username || '-' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="房主ID">
        {{ room.owner?.id || '-' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="成员数量">
        {{ room.member_count }} / {{ room.max_members }}
      </NDescriptionsItem>
      <NDescriptionsItem label="创建时间">
        {{ formatTime(room.created_at) }}
      </NDescriptionsItem>
      <NDescriptionsItem label="最后更新" :span="2">
        {{ formatTime(room.updated_at) }}
      </NDescriptionsItem>
    </NDescriptions>

    <div class="modal-actions">
      <NSpace justify="end">
        <NButton type="primary" @click="handleViewMessages">
          <template #icon>
            <MessageSquare :size="16" />
          </template>
          查看消息
        </NButton>
        <NButton @click="handleViewAnalytics">
          <template #icon>
            <BarChart3 :size="16" />
          </template>
          数据分析
        </NButton>
        <NButton @click="handleEdit">
          <template #icon>
            <Edit :size="16" />
          </template>
          编辑
        </NButton>
      </NSpace>
    </div>
  </div>
</template>

<style scoped>
.room-detail-modal {
  padding: 8px 0;
}

.modal-actions {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}
</style>
