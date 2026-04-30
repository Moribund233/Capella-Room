<script setup lang="ts">
import { NTag, NSpace, NDescriptions, NDescriptionsItem, NDivider, NAvatar } from 'naive-ui'
import type { AdminMessageInfo } from '@/api/admin'

/**
 * 组件属性定义
 */
interface Props {
  /** 消息信息 */
  message: AdminMessageInfo | null
}

defineProps<Props>()

/**
 * 消息类型映射配置
 */
const messageTypeConfig: Record<string, { text: string; type: 'default' | 'info' | 'success' | 'warning' }> = {
  text: { text: '文本', type: 'default' },
  image: { text: '图片', type: 'info' },
  file: { text: '文件', type: 'success' },
  system: { text: '系统', type: 'warning' },
}

/**
 * 格式化日期时间
 * @param dateStr ISO 8601 格式日期字符串
 * @returns 格式化后的日期时间字符串
 */
const formatDateTime = (dateStr: string | undefined | null): string => {
  if (!dateStr) return '-'
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div v-if="message" class="message-detail-modal">
    <!-- 消息状态头部 -->
    <div class="message-header">
      <NSpace size="small">
        <NTag
          :type="messageTypeConfig[message.message_type]?.type || 'default'"
          size="large"
        >
          {{ messageTypeConfig[message.message_type]?.text || '未知' }}
        </NTag>
        <NTag v-if="message.is_deleted" type="error" size="large">
          已删除
        </NTag>
        <NTag v-else type="success" size="large">
          正常
        </NTag>
      </NSpace>
    </div>

    <NDivider />

    <!-- 发送者信息 -->
    <div class="sender-section">
      <h4 class="section-title">发送者信息</h4>
      <div class="sender-info">
        <NAvatar
          :src="message.sender?.avatar_url || undefined"
          :fallback-src="`https://api.dicebear.com/7.x/avataaars/svg?seed=${message.sender?.username || 'user'}`"
          :size="50"
          round
          class="sender-avatar"
        />
        <div class="sender-details">
          <div class="sender-name">{{ message.sender?.username || '-' }}</div>
          <div class="sender-id">ID: {{ message.sender?.id || '-' }}</div>
        </div>
      </div>
    </div>

    <NDivider />

    <!-- 消息详细信息 -->
    <NDescriptions :columns="1" label-placement="left" label-align="right" label-style="width: 100px">
      <NDescriptionsItem label="消息ID">
        <span class="copyable-text">{{ message.id }}</span>
      </NDescriptionsItem>
      <NDescriptionsItem label="房间ID">
        {{ message.room_id }}
      </NDescriptionsItem>
      <NDescriptionsItem label="消息类型">
        <NTag :type="messageTypeConfig[message.message_type]?.type || 'default'" size="small">
          {{ messageTypeConfig[message.message_type]?.text || '未知' }}
        </NTag>
      </NDescriptionsItem>
      <NDescriptionsItem label="回复消息">
        {{ message.reply_to ? `消息ID: ${message.reply_to}` : '无' }}
      </NDescriptionsItem>
      <NDescriptionsItem label="编辑次数">
        {{ message.edit_count || 0 }}
      </NDescriptionsItem>
      <NDescriptionsItem label="最后编辑">
        {{ formatDateTime(message.edited_at) }}
      </NDescriptionsItem>
      <NDescriptionsItem label="发送时间">
        {{ formatDateTime(message.created_at) }}
      </NDescriptionsItem>
    </NDescriptions>

    <NDivider />

    <!-- 消息内容 -->
    <div class="content-section">
      <h4 class="section-title">消息内容</h4>
      <div class="message-content" :class="{ 'is-deleted': message.is_deleted }">
        {{ message.content }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.message-detail-modal {
  padding: 8px;
}

.message-header {
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.section-title {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.sender-section {
  margin-bottom: 16px;
}

.sender-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sender-avatar {
  flex-shrink: 0;
}

.sender-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sender-name {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary);
}

.sender-id {
  font-size: 12px;
  color: var(--text-secondary);
}

.content-section {
  margin-top: 16px;
}

.message-content {
  padding: 16px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
}

.message-content.is-deleted {
  text-decoration: line-through;
  color: var(--text-secondary);
}

.copyable-text {
  font-family: monospace;
  font-size: 12px;
  color: var(--text-secondary);
}

:deep(.n-descriptions-table-content) {
  font-size: 13px;
}
</style>
