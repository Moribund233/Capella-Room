<script setup lang="ts">
import { ref } from 'vue'
import {
  Send,
  Search,
  RefreshCw,
  MessageSquare,
  Trash2,
  Edit,
  Reply,
  History,
  User
} from 'lucide-vue-next'

const selectedRoom = ref('lobby')
const messageContent = ref('')
const searchQuery = ref('')

const rooms = [
  { label: '大厅', value: 'lobby' },
  { label: '技术交流', value: 'tech' },
  { label: '测试房间', value: 'test' }
]

const messages = ref([
  {
    id: '1',
    content: '大家好！欢迎来到 Seredeli Room！',
    sender: 'admin',
    sender_id: '1',
    room_id: 'lobby',
    created_at: '2024-03-10 10:00:00',
    type: 'text'
  },
  {
    id: '2',
    content: '这个项目看起来很棒！',
    sender: 'user_123',
    sender_id: '2',
    room_id: 'lobby',
    created_at: '2024-03-10 10:05:00',
    type: 'text'
  },
  {
    id: '3',
    content: 'WebSocket 功能测试成功',
    sender: 'test_user',
    sender_id: '3',
    room_id: 'lobby',
    created_at: '2024-03-10 10:10:00',
    type: 'text'
  },
  {
    id: '4',
    content: '有人知道如何部署到生产环境吗？',
    sender: 'user_456',
    sender_id: '4',
    room_id: 'lobby',
    created_at: '2024-03-10 10:15:00',
    type: 'text'
  }
])

const sendMessage = () => {
  if (!messageContent.value) return
  messages.value.push({
    id: String(messages.value.length + 1),
    content: messageContent.value,
    sender: 'current_user',
    sender_id: '99',
    room_id: selectedRoom.value,
    created_at: new Date().toLocaleString(),
    type: 'text'
  })
  messageContent.value = ''
}

const deleteMessage = (msg: any) => {
  const index = messages.value.findIndex((m) => m.id === msg.id)
  if (index > -1) {
    messages.value.splice(index, 1)
  }
}
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
            <n-select v-model:value="selectedRoom" :options="rooms" style="width: 150px" />
            <n-button text>
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
          <div
            v-for="msg in messages"
            :key="msg.id"
            style="margin-bottom: var(--space-md); padding: var(--space-md); background-color: var(--bg-white); border-radius: var(--radius-md)"
          >
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-sm)">
              <n-space align="center">
                <n-avatar size="small" :style="{ backgroundColor: 'var(--primary)' }">
                  {{ msg.sender.charAt(0).toUpperCase() }}
                </n-avatar>
                <span style="font-weight: 500">{{ msg.sender }}</span>
                <span style="font-size: 12px; color: var(--text-muted)">{{ msg.created_at }}</span>
              </n-space>
              <n-space>
                <n-button size="tiny" text>
                  <template #icon>
                    <Reply class="icon-sm" />
                  </template>
                </n-button>
                <n-button size="tiny" text>
                  <template #icon>
                    <Edit class="icon-sm" />
                  </template>
                </n-button>
                <n-button size="tiny" text type="error" @click="deleteMessage(msg)">
                  <template #icon>
                    <Trash2 class="icon-sm" />
                  </template>
                </n-button>
              </n-space>
            </div>
            <div style="padding-left: 36px; color: var(--text-primary)">{{ msg.content }}</div>
          </div>
        </div>

        <!-- 发送消息 -->
        <n-input-group>
          <n-input
            v-model:value="messageContent"
            placeholder="输入消息内容..."
            @keyup.enter="sendMessage"
          />
          <n-button type="primary" @click="sendMessage">
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
          >
            <template #prefix>
              <Search class="icon-sm" />
            </template>
          </n-input>
          <n-button type="primary" block>
            <template #icon>
              <Search class="icon-sm" />
            </template>
            搜索
          </n-button>
        </n-card>

        <n-card title="搜索选项">
          <n-form label-placement="left" label-width="80">
            <n-form-item label="房间">
              <n-select v-model:value="selectedRoom" :options="rooms" />
            </n-form-item>
            <n-form-item label="用户">
              <n-input placeholder="用户名" />
            </n-form-item>
            <n-form-item label="时间范围">
              <n-date-picker type="daterange" style="width: 100%" />
            </n-form-item>
            <n-form-item label="消息类型">
              <n-checkbox-group>
                <n-space>
                  <n-checkbox label="文本" value="text" checked />
                  <n-checkbox label="图片" value="image" />
                  <n-checkbox label="文件" value="file" />
                </n-space>
              </n-checkbox-group>
            </n-form-item>
          </n-form>
        </n-card>

        <n-card title="操作">
          <n-space vertical style="width: 100%">
            <n-button block>
              <template #icon>
                <History class="icon-sm" />
              </template>
              查看历史记录
            </n-button>
            <n-button block>
              <template #icon>
                <Trash2 class="icon-sm" />
              </template>
              批量删除
            </n-button>
          </n-space>
        </n-card>
      </div>
    </div>
  </div>
</template>
