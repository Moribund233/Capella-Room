<template>
  <div class="home-welcome">
    <div class="welcome-content">
      <div class="welcome-avatar">
        <img v-if="userAvatar" :src="userAvatar" class="avatar-img" :alt="username" />
        <div v-else class="avatar-fallback">
          <span class="avatar-text">{{ usernameFirstChar }}</span>
        </div>
        <div class="status-indicator" :class="connectionStatusClass"></div>
      </div>
      <div class="welcome-text">
        <n-h2 class="welcome-title">
          欢迎回来，<n-text type="primary">{{ username }}</n-text>
        </n-h2>
        <n-text class="welcome-subtitle" depth="3">
          {{ currentDate }} {{ currentWeekday }}
        </n-text>
      </div>
    </div>
    <n-tag :type="connectionType" size="large" round>
      <template #icon>
        <n-icon :component="connectionIcon" />
      </template>
      {{ connectionText }}
    </n-tag>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NH2, NText, NTag, NIcon } from 'naive-ui'
import { Wifi, WifiOff, Loader2 } from 'lucide-vue-next'
import { useAuthStore, useWebSocketStore } from '@/store'

const authStore = useAuthStore()
const wsStore = useWebSocketStore()

const username = computed(() => authStore.username || '访客')

const usernameFirstChar = computed(() => {
  return username.value.charAt(0).toUpperCase()
})

const userAvatar = computed(() => {
  return authStore.user?.avatar_url || null
})

const connectionStatusClass = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return 'status-online'
    case 'connecting':
    case 'reconnecting':
      return 'status-busy'
    default:
      return 'status-offline'
  }
})

const now = new Date()
const currentDate = computed(() => {
  return now.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  })
})

const currentWeekday = computed(() => {
  const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  return weekdays[now.getDay()]
})

const connectionType = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return 'success'
    case 'connecting':
    case 'reconnecting':
      return 'warning'
    default:
      return 'error'
  }
})

const connectionText = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return '已连接'
    case 'connecting':
      return '连接中...'
    case 'reconnecting':
      return '重连中...'
    default:
      return '未连接'
  }
})

const connectionIcon = computed(() => {
  switch (wsStore.status) {
    case 'connected':
      return Wifi
    case 'connecting':
    case 'reconnecting':
      return Loader2
    default:
      return WifiOff
  }
})
</script>

<style scoped>
.home-welcome {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px;
  background: var(--card-color);
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.welcome-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.welcome-avatar {
  position: relative;
  width: 64px;
  height: 64px;
}

.avatar-img {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  object-fit: cover;
  border: 2px solid var(--border-color);
}

.avatar-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  background: var(--header-bg);
  border-radius: 16px;
  border: 2px solid var(--border-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.avatar-text {
  font-size: 28px;
  font-weight: 600;
  color: var(--primary-color);
}

.status-indicator {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 3px solid var(--card-color);
  transition: background-color 0.3s ease;
}

.status-online {
  background-color: var(--success-color, #18a058);
}

.status-busy {
  background-color: var(--warning-color, #f0a020);
}

.status-offline {
  background-color: var(--error-color, #d03050);
}

.welcome-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.welcome-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.welcome-subtitle {
  font-size: 14px;
}

@media (max-width: 640px) {
  .home-welcome {
    flex-direction: column;
    gap: 16px;
    text-align: center;
  }

  .welcome-content {
    flex-direction: column;
  }
}
</style>
