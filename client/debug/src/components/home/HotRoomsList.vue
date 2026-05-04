<template>
  <n-card title="热门房间" :bordered="false" class="hot-rooms-card">
    <template #header-extra>
      <n-button text type="primary" size="small" @click="goToRooms">
        查看全部
        <template #icon>
          <n-icon :component="ChevronRight" />
        </template>
      </n-button>
    </template>

    <n-list hoverable clickable>
      <n-list-item
        v-for="(room, index) in hotRooms"
        :key="room.id"
        @click="enterRoom(room.id)"
      >
        <n-thing>
          <template #avatar>
            <n-avatar
              :style="{ background: getRankColor(index) }"
              :size="32"
            >
              {{ index + 1 }}
            </n-avatar>
          </template>
          <template #header>
            <n-space align="center" :size="8">
              <n-text strong>{{ room.name }}</n-text>
              <n-tag v-if="index < 3" :type="getRankType(index)" size="small" round>
                {{ getRankLabel(index) }}
              </n-tag>
            </n-space>
          </template>
          <template #description>
            <n-space :size="16">
              <n-text depth="3" class="room-meta">
                <n-icon :component="Users" :size="14" />
                {{ room.onlineCount }} 人在线
              </n-text>
              <n-text depth="3" class="room-meta">
                <n-icon :component="MessageCircle" :size="14" />
                {{ room.messageCount }} 条消息
              </n-text>
            </n-space>
          </template>
        </n-thing>
      </n-list-item>
    </n-list>

    <n-empty v-if="hotRooms.length === 0" description="暂无热门房间" />
  </n-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  NCard,
  NList,
  NListItem,
  NThing,
  NAvatar,
  NSpace,
  NText,
  NTag,
  NButton,
  NIcon,
  NEmpty,
} from 'naive-ui'
import { ChevronRight, Users, MessageCircle } from 'lucide-vue-next'

interface HotRoom {
  id: string
  name: string
  onlineCount: number
  messageCount: number
}

const router = useRouter()

// 模拟热门房间数据
const hotRooms = ref<HotRoom[]>([
  { id: '1', name: 'Rust 技术交流', onlineCount: 128, messageCount: 3456 },
  { id: '2', name: 'Vue3 开发者社区', onlineCount: 96, messageCount: 2890 },
  { id: '3', name: '闲聊灌水区', onlineCount: 85, messageCount: 5678 },
  { id: '4', name: '游戏开黑大厅', onlineCount: 72, messageCount: 1234 },
  { id: '5', name: '前端学习小组', onlineCount: 56, messageCount: 890 },
])

const getRankColor = (index: number) => {
  const colors = ['#f0a020', '#8c8c8c', '#8b4513', '#2080f0', '#18a058']
  return colors[index] || '#2080f0'
}

const getRankType = (index: number) => {
  const types = ['warning', 'default', 'success'] as const
  return types[index] || 'default'
}

const getRankLabel = (index: number) => {
  const labels = ['🔥 热门', '⭐ 推荐', '👍 活跃']
  return labels[index] || ''
}

const goToRooms = () => {
  router.push('/rooms')
}

const enterRoom = (roomId: string) => {
  router.push(`/rooms/${roomId}`)
}
</script>

<style scoped>
.hot-rooms-card {
  height: 100%;
}

.room-meta {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
}
</style>
