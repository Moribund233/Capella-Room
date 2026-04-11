<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  Send,
  Search,
  RefreshCw,
  MessageSquare,
  Trash2,
  Reply
} from 'lucide-vue-next'
import { getRoomMessages, deleteMessage, searchMessages, getRooms, type Room } from '@/api'
import type { Message } from '@/api/message'
import { useWebSocketStore } from '@/stores/websocket'
import { storeToRefs } from 'pinia'

const message = useMessage()
const wsStore = useWebSocketStore()
const { isConnected: wsConnected } = storeToRefs(wsStore)

// ========== 状态 ==========
const rooms = ref<Room[]>([])
const selectedRoomId = ref<string>('')
const messages = ref<Message[]>([])
const loading = ref(false)
const messageContent = ref('')
const searchQuery = ref('')
const searchResults = ref<Message[]>([])
const showSearchPanel = ref(false)
const hasMore = ref(false)
const replyToMessage = ref<Message | null>(null)

// ========== 数据加载 ==========
const loadRooms = async () => {
  try {
    const data = await getRooms()
    rooms.value = data
    if (data.length > 0 && !selectedRoomId.value) {
      const firstRoom = data[0]
      if (firstRoom && firstRoom.id) {
        selectedRoomId.value = firstRoom.id
      }
    }
  } catch (error) {
    message.error('加载房间列表失败')
    console.error(error)
  }
}

const loadMessages = async () => {
  if (!selectedRoomId.value) return

  loading.value = true
  try {
    const result = await getRoomMessages(selectedRoomId.value, { limit: 50 })
    messages.value = result.messages
    hasMore.value = result.has_more
  } catch (error) {
    message.error('加载消息失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// ========== 发送消息 ==========
const handleSendMessage = async () => {
  if (!messageContent.value.trim() || !selectedRoomId.value) return

  if (!wsConnected.value) {
    message.error('WebSocket 未连接，无法发送消息')
    return
  }

  try {
    // 通过 WebSocket 发送消息
    const success = wsStore.send({
      type: 'ChatMessage',
      payload: {
        room_id: selectedRoomId.value,
        content: messageContent.value.trim(),
        reply_to: replyToMessage.value?.id || null
      }
    })

    if (success) {
      messageContent.value = ''
      replyToMessage.value = null
      message.success('消息已发送')
      // 延迟后刷新消息列表
      setTimeout(() => loadMessages(), 500)
    } else {
      message.error('发送消息失败，请检查 WebSocket 连接')
    }
  } catch (error) {
    message.error('发送消息失败')
    console.error(error)
  }
}

// ========== 删除消息 ==========
const handleDeleteMessage = async (msg: Message) => {
  try {
    await deleteMessage(msg.id)
    message.success('消息已删除')
    await loadMessages()
  } catch (error) {
    message.error('删除消息失败')
    console.error(error)
  }
}

// ========== 搜索消息 ==========
const handleSearch = async () => {
  if (!searchQuery.value.trim()) {
    showSearchPanel.value = false
    return
  }

  try {
    const result = await searchMessages({
      query: searchQuery.value.trim(),
      room_id: selectedRoomId.value,
    })
    searchResults.value = result.messages
    showSearchPanel.value = true
  } catch (error) {
    message.error('搜索消息失败')
    console.error(error)
  }
}

// ========== 回复消息 ==========
const handleReply = (msg: Message) => {
  replyToMessage.value = msg
}

const cancelReply = () => {
  replyToMessage.value = null
}

// ========== 监听房间变化 ==========
watch(selectedRoomId, () => {
  loadMessages()
  showSearchPanel.value = false
  searchQuery.value = ''
})

// ========== 初始化 ==========
onMounted(() => {
  loadRooms()
  // 连接 WebSocket（如果未连接）
  if (!wsConnected.value) {
    wsStore.connect()
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
        消息测试
      </h1>
      <p class="page-subtitle">测试消息发送、接收和搜索功能</p>
    </div>

    <div style="display: grid; grid-template-columns: 1fr 350px; gap: var(--space-lg)">
      <!-- 左侧：消息区域 -->
      <n-card title="消息测试">
        <template #header-extra>
          <n-space>
            <n-select
              v-model:value="selectedRoomId"
              :options="rooms.map(r => ({ label: r.name, value: r.id }))"
              placeholder="选择房间"
              style="width: 180px"
            />
            <n-button text @click="loadMessages">
              <template #icon>
                <RefreshCw class="icon-sm" />
              </template>
            </n-button>
          </n-space>
        </template>

        <!-- 消息列表 -->
        <div
          style="
            min-height: 400px;
            max-height: 500px;
            overflow-y: auto;
            padding: var(--space-md);
            background-color: var(--bg-secondary);
            border-radius: var(--radius-md);
            margin-bottom: var(--space-md);
          "
        >
          <div v-if="loading" style="text-align: center; padding: var(--space-xl)">
            <n-spin size="medium" />
          </div>
          <div v-else-if="messages.length === 0" style="text-align: center; padding: var(--space-xl); color: var(--text-muted)">
            暂无消息
          </div>
          <div
            v-for="msg in messages"
            :key="msg.id"
            style="margin-bottom: var(--space-md); padding: var(--space-md); background-color: var(--bg-white); border-radius: var(--radius-md)"
          >
            <!-- 回复引用 -->
            <div v-if="msg.reply_to" style="margin-bottom: var(--space-sm); padding: var(--space-sm); background-color: var(--bg-secondary); border-radius: var(--radius-sm); font-size: 12px; color: var(--text-muted)">
              <Reply class="icon-sm" style="display: inline; vertical-align: middle; margin-right: 4px" />
              回复消息
            </div>
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-sm)">
              <n-space align="center">
                <n-avatar size="small" :style="{ backgroundColor: 'var(--primary)' }">
                  {{ msg.sender.charAt(0).toUpperCase() }}
                </n-avatar>
                <span style="font-weight: 500">{{ msg.sender }}</span>
                <span style="font-size: 12px; color: var(--text-muted)">{{ new Date(msg.created_at).toLocaleString() }}</span>
              </n-space>
              <n-space>
                <n-button size="tiny" text @click="handleReply(msg)">
                  <template #icon>
                    <Reply class="icon-sm" />
                  </template>
                </n-button>
                <n-button size="tiny" text type="error" @click="handleDeleteMessage(msg)">
                  <template #icon>
                    <Trash2 class="icon-sm" />
                  </template>
                </n-button>
              </n-space>
            </div>
            <div style="padding-left: 36px; color: var(--text-primary)">{{ msg.content }}</div>
          </div>
        </div>

        <!-- 回复提示 -->
        <div v-if="replyToMessage" style="margin-bottom: var(--space-sm); padding: var(--space-sm); background-color: var(--bg-secondary); border-radius: var(--radius-sm); display: flex; justify-content: space-between; align-items: center">
          <span style="font-size: 13px; color: var(--text-muted)">
            <Reply class="icon-sm" style="display: inline; vertical-align: middle; margin-right: 4px" />
            回复 {{ replyToMessage.sender }}: {{ replyToMessage.content.substring(0, 30) }}{{ replyToMessage.content.length > 30 ? '...' : '' }}
          </span>
          <n-button size="tiny" text @click="cancelReply">
            <template #icon>
              <Trash2 class="icon-sm" />
            </template>
          </n-button>
        </div>

        <!-- 发送消息 -->
        <n-input-group>
          <n-input
            v-model:value="messageContent"
            placeholder="输入消息内容..."
            @keyup.enter="handleSendMessage"
          />
          <n-button type="primary" @click="handleSendMessage">
            <template #icon>
              <Send class="icon-sm" />
            </template>
            发送
          </n-button>
        </n-input-group>
      </n-card>

      <!-- 右侧：搜索和历史 -->
      <div style="display: flex; flex-direction: column; gap: var(--space-lg)">
        <n-card title="消息搜索">
          <n-input
            v-model:value="searchQuery"
            placeholder="搜索消息内容..."
            style="margin-bottom: var(--space-md)"
            @keyup.enter="handleSearch"
          >
            <template #prefix>
              <Search class="icon-sm" />
            </template>
          </n-input>
          <n-button type="primary" block @click="handleSearch">
            <template #icon>
              <Search class="icon-sm" />
            </template>
            搜索
          </n-button>
        </n-card>

        <!-- 搜索结果 -->
        <n-card v-if="showSearchPanel" title="搜索结果">
          <div style="max-height: 400px; overflow-y: auto">
            <div v-if="searchResults.length === 0" style="text-align: center; padding: var(--space-lg); color: var(--text-muted)">
              未找到匹配的消息
            </div>
            <div
              v-for="msg in searchResults"
              :key="msg.id"
              style="margin-bottom: var(--space-md); padding: var(--space-md); background-color: var(--bg-secondary); border-radius: var(--radius-md)"
            >
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-xs)">
                <span style="font-weight: 500; font-size: 13px">{{ msg.sender }}</span>
                <span style="font-size: 11px; color: var(--text-muted)">{{ new Date(msg.created_at).toLocaleString() }}</span>
              </div>
              <div style="color: var(--text-primary); font-size: 13px">{{ msg.content }}</div>
            </div>
          </div>
        </n-card>

        <n-card title="房间信息">
          <n-descriptions :column="1" size="small">
            <n-descriptions-item label="当前房间">
              {{ rooms.find(r => r.id === selectedRoomId)?.name || '未选择' }}
            </n-descriptions-item>
            <n-descriptions-item label="消息数量">
              {{ messages.length }}
            </n-descriptions-item>
            <n-descriptions-item label="在线成员">
              {{ rooms.find(r => r.id === selectedRoomId)?.member_count || 0 }}
            </n-descriptions-item>
          </n-descriptions>
        </n-card>
      </div>
    </div>
  </div>
</template>
