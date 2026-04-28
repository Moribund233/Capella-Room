<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { NCard, NPagination, NButton, NInput, NSpace, NTag, NList, NListItem, NThing, NText, NTime, NAvatar, NEmpty, useMessage, useDialog } from 'naive-ui'
import { ArrowLeft, Trash2, Search } from 'lucide-vue-next'
import { useRoomStore } from '@/stores'
import type { AdminMessageInfo } from '@/api/admin'

const route = useRoute()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()

/**
 * 使用房间管理 Store
 */
const roomStore = useRoomStore()

// 使用 storeToRefs 保持响应性
const {
  currentRoom,
  messages,
  messagesTotal,
  messagesPage,
  messagesPageSize,
  messagesLoading,
} = storeToRefs(roomStore)

/** 搜索关键词 */
const searchKeyword = ref('')

/** 当前房间ID */
const roomId = computed(() => route.params.id as string)

/** 房间名称 */
const roomName = computed(() => currentRoom.value?.name || '未知房间')

/**
 * 加载消息列表
 */
const loadMessages = async () => {
  if (!roomId.value) return
  await roomStore.fetchRoomMessages(roomId.value, {
    page: messagesPage.value,
    pageSize: messagesPageSize.value,
    search: searchKeyword.value || undefined,
  })
}

/**
 * 处理搜索
 */
const handleSearch = () => {
  loadMessages()
}

/**
 * 处理分页变化
 */
const handlePageChange = (page: number, pageSize: number) => {
  roomStore.fetchRoomMessages(roomId.value, {
    page,
    pageSize,
    search: searchKeyword.value || undefined,
  })
}

/**
 * 处理删除消息
 */
const handleDeleteMessage = (msg: AdminMessageInfo) => {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除这条消息吗？\n\n发送者: ${msg.sender.username}\n内容: ${msg.content.substring(0, 50)}${msg.content.length > 50 ? '...' : ''}`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      const success = await roomStore.deleteMessage(msg.id)
      if (success) {
        message.success('消息已删除')
      }
    },
  })
}

/**
 * 获取消息类型标签
 */
const getMessageTypeTag = (type: string) => {
  const typeMap: Record<string, { type: 'default' | 'primary' | 'success' | 'warning' | 'error'; label: string }> = {
    text: { type: 'default', label: '文本' },
    image: { type: 'success', label: '图片' },
    file: { type: 'warning', label: '文件' },
    system: { type: 'error', label: '系统' },
  }
  return typeMap[type] || { type: 'default', label: type }
}

/**
 * 返回上一页
 */
const handleBack = () => {
  router.back()
}

// 页面加载时获取数据
onMounted(async () => {
  if (roomId.value) {
    // 如果当前房间未加载，先加载房间详情
    if (!currentRoom.value || currentRoom.value.id !== roomId.value) {
      await roomStore.fetchRoomDetail(roomId.value)
    }
    await loadMessages()
  }
})
</script>

<template>
  <div class="room-messages-page">
    <div class="page-header">
      <div class="header-left">
        <NButton quaternary circle @click="handleBack">
          <template #icon>
            <ArrowLeft :size="20" />
          </template>
        </NButton>
        <div class="header-title">
          <h1 class="page-title">消息管理</h1>
          <p class="page-subtitle">房间: {{ roomName }}</p>
        </div>
      </div>
    </div>

    <NCard class="search-card" :bordered="false">
      <NSpace align="center">
        <NInput
          v-model:value="searchKeyword"
          placeholder="搜索消息内容..."
          clearable
          style="width: 300px"
          @keyup.enter="handleSearch"
        >
          <template #prefix>
            <Search :size="16" />
          </template>
        </NInput>
        <NButton type="primary" @click="handleSearch">搜索</NButton>
      </NSpace>
    </NCard>

    <NCard class="messages-card" :bordered="false" :loading="messagesLoading">
      <NEmpty v-if="!messages || messages.length === 0" description="暂无消息" />
      <NList v-else class="message-list" hoverable clickable>
        <NListItem
          v-for="msg in messages"
          :key="msg.id"
          class="message-item"
          :class="{ 'is-deleted': msg.is_deleted }"
        >
          <NThing>
            <template #avatar>
              <NAvatar
                v-if="msg.sender.avatar_url"
                :src="msg.sender.avatar_url"
                :size="40"
                round
              />
              <NAvatar v-else :size="40" round>
                {{ msg.sender.username.charAt(0).toUpperCase() }}
              </NAvatar>
            </template>
            <template #header>
              <NSpace align="center" :size="8">
                <NText strong>{{ msg.sender.username }}</NText>
                <NTag size="small" :type="getMessageTypeTag(msg.message_type).type">
                  {{ getMessageTypeTag(msg.message_type).label }}
                </NTag>
                <NTime :time="new Date(msg.created_at)" type="datetime" />
                <NText v-if="msg.edit_count > 0" type="info" depth="3" style="font-size: 12px">
                  已编辑 {{ msg.edit_count }} 次
                </NText>
              </NSpace>
            </template>
            <template #header-extra>
              <NButton
                v-if="!msg.is_deleted"
                size="small"
                type="error"
                quaternary
                circle
                @click="handleDeleteMessage(msg)"
              >
                <template #icon>
                  <Trash2 :size="16" />
                </template>
              </NButton>
            </template>
            <template #description>
              <NText
                :type="msg.is_deleted ? 'error' : 'default'"
                :delete="msg.is_deleted"
                style="white-space: pre-wrap; word-break: break-word"
              >
                {{ msg.is_deleted ? '[已删除]' : msg.content }}
              </NText>
            </template>
          </NThing>
        </NListItem>
      </NList>

      <div v-if="messagesTotal > 0" class="pagination-wrapper">
        <NPagination
          :page="messagesPage"
          :page-size="messagesPageSize"
          :item-count="messagesTotal"
          :page-sizes="[20, 50, 100]"
          show-size-picker
          @update:page="handlePageChange($event, messagesPageSize)"
          @update:page-size="handlePageChange(1, $event)"
        />
      </div>
    </NCard>
  </div>
</template>

<style scoped>
.room-messages-page {
  min-height: 100%;
}

.page-header {
  margin-bottom: 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title {
  display: flex;
  flex-direction: column;
}

.page-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.page-subtitle {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 4px 0 0 0;
}

.search-card {
  margin-bottom: 16px;
}

.messages-card {
  min-height: 400px;
}

.message-list {
  --n-padding-left: 16px;
  --n-padding-right: 16px;
}

.message-item.is-deleted {
  opacity: 0.6;
  background: var(--disabled-color);
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

/* 移动端适配 */
@media screen and (max-width: 768px) {
  .page-title {
    font-size: 20px;
  }
}
</style>
