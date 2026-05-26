<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NavBar } from '@/components/nav'
import { QuickBar } from '@/components/quick'
import type { QuickItem } from '@/components/quick'
import {
  Search,
  Setting,
  Microphone,
  MoreFilled,
  Plus,
  Lock,
  ArrowRight,
  Bell,
  Moon,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()

// QuickBar 配置
const quickItems = ref<QuickItem[]>([
  {
    key: 'notifications',
    display: 'visible',
    icon: Bell,
    label: t('profile.preferences.notifications.title'),
    badge: 3,
    onClick: () => {
      // TODO: 打开通知面板
    },
  },
  {
    key: 'theme',
    display: 'visible',
    icon: Moon,
    label: t('profile.appearance.theme.title'),
    onClick: () => {
      // TODO: 切换主题
    },
  },
])

// 当前选中的频道
const activeChannel = ref('general')

// 频道列表
const channelCategories = [
  {
    name: 'Channels',
    channels: [
      { id: 'general', name: 'general', unread: 3 },
      { id: 'introductions', name: 'introductions', unread: 0 },
      { id: 'random', name: 'random', unread: 0 },
      { id: 'show-and-tell', name: 'show-and-tell', unread: 0 },
      { id: 'events', name: 'events', unread: 5 },
    ],
  },
  {
    name: 'Projects',
    channels: [
      { id: 'design-system', name: 'design-system', unread: 0 },
      { id: 'frontend', name: 'frontend', unread: 0 },
      { id: 'backend', name: 'backend', unread: 0, locked: true },
      { id: 'product-feedback', name: 'product-feedback', unread: 0 },
    ],
  },
]

// 消息列表
const messages = [
  {
    id: 1,
    author: 'alex',
    avatar: 'A',
    avatarColor: 'var(--wave-accent)',
    time: '9:42 AM',
    content: 'Hey team! 🎉 Just pushed the new design system updates to staging. Would love for everyone to take a look before we ship it live.',
    reactions: [
      { emoji: '🎉', count: 6 },
      { emoji: '🔥', count: 3 },
    ],
    threadCount: 5,
  },
  {
    id: 2,
    author: 'jordan',
    avatar: 'J',
    avatarColor: 'var(--wave-accent-green)',
    time: '9:48 AM',
    content: 'Looking great! I noticed the button component\'s hover state is a bit subtle — might want to bump the contrast there. Otherwise ship it! 🚀',
    reactions: [
      { emoji: '👍', count: 3 },
    ],
  },
  {
    id: 3,
    author: 'mira',
    avatar: 'M',
    avatarColor: 'var(--wave-accent-pink)',
    time: '9:55 AM',
    content: 'Agreed with @jordan! Also the spacing on mobile cards feels a bit tight. I left a comment in Figma.',
    reactions: [
      { emoji: '👀', count: 4 },
    ],
  },
  {
    id: 4,
    author: 'kaito',
    avatar: 'K',
    avatarColor: 'var(--wave-accent-blue)',
    time: '10:12 AM',
    content: 'Tested on staging — looks solid on Chrome and Safari. There\'s a minor layout shift on Firefox at 1024px, but I think it\'s just a missing @supports query. I can patch that today.',
    reactions: [],
  },
]

// 成员列表
const memberGroups = [
  {
    label: 'Online',
    members: [
      { name: 'alex', color: 'var(--wave-accent)', status: 'online' },
      { name: 'jordan', color: 'var(--wave-accent-green)', status: 'online' },
      { name: 'mira', color: 'var(--wave-accent-pink)', status: 'online' },
      { name: 'kaito', color: 'var(--wave-accent-blue)', status: 'online' },
      { name: 'taylor', color: 'var(--wave-accent-orange)', status: 'online' },
    ],
  },
  {
    label: 'Offline',
    members: [
      { name: 'sam', color: 'var(--wave-accent)', status: 'offline' },
      { name: 'riley', color: 'var(--wave-accent-green)', status: 'offline' },
    ],
  },
]

// 输入框内容
const messageInput = ref('')

/**
 * 选择频道
 * @param channelId - 频道ID
 */
function selectChannel(channelId: string) {
  activeChannel.value = channelId
}

/**
 * 跳转到个人资料
 */
function goToProfile() {
  router.push('/profile')
}

/**
 * 跳转到线程页面
 */
function goToThread() {
  router.push('/thread')
}

/**
 * 发送消息
 */
function sendMessage() {
  if (!messageInput.value.trim()) return
  // TODO: 发送消息逻辑
  messageInput.value = ''
}
</script>

<template>
  <div class="app-layout">
    <!-- 窄边导航栏 -->
    <NavBar>
      <template #quick-bar>
        <QuickBar :items="quickItems" />
      </template>
    </NavBar>

    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-header" @click="$router.push('/')">
        <span>Wave Community</span>
        <el-icon><ArrowRight /></el-icon>
      </div>

      <div class="sidebar-search">
        <el-input
          :placeholder="t('chat.findChannel')"
          :prefix-icon="Search"
          size="small"
        />
      </div>

      <div class="channels">
        <div
          v-for="category in channelCategories"
          :key="category.name"
          class="channel-category"
        >
          <div class="category-header">
            <span>{{ category.name }}</span>
            <el-icon class="add-icon"><Plus /></el-icon>
          </div>
          <div
            v-for="channel in category.channels"
            :key="channel.id"
            class="channel"
            :class="{ active: activeChannel === channel.id }"
            @click="selectChannel(channel.id)"
          >
            <span class="channel-prefix">
              <el-icon v-if="channel.locked"><Lock /></el-icon>
              <span v-else>#</span>
            </span>
            <span class="channel-name">{{ channel.name }}</span>
            <span v-if="channel.unread > 0" class="channel-badge">
              {{ channel.unread }}
            </span>
          </div>
        </div>
      </div>

      <!-- 用户信息 -->
      <div class="user-section" @click="goToProfile">
        <div class="user-avatar">
          <span>A</span>
          <span class="status-dot"></span>
        </div>
        <div class="user-info">
          <div class="user-name">alex</div>
          <div class="user-status">{{ t('chat.online') }}</div>
        </div>
        <div class="user-controls">
          <el-button text circle size="small">
            <el-icon><Microphone /></el-icon>
          </el-button>
          <el-button text circle size="small">
            <el-icon><Setting /></el-icon>
          </el-button>
        </div>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="main">
      <!-- 聊天头部 -->
      <header class="chat-header">
        <div class="channel-info">
          <span class="channel-hash">#</span>
          <span class="channel-title">{{ activeChannel }}</span>
          <span class="channel-topic">· Community updates & announcements</span>
        </div>
        <div class="chat-header-right">
          <div class="member-avatars">
            <div
              v-for="i in 5"
              :key="i"
              class="mini-avatar"
              :style="{
                background: [
                  'var(--wave-accent)',
                  'var(--wave-accent-pink)',
                  'var(--wave-accent-green)',
                  'var(--wave-accent-orange)',
                  'var(--wave-accent-blue)',
                ][i - 1],
              }"
            >
              {{ ['A', 'M', 'J', 'T', 'K'][i - 1] }}
            </div>
          </div>
          <span class="member-count">18 {{ t('chat.online') }}</span>
          <el-button text circle>
            <el-icon><Search /></el-icon>
          </el-button>
          <el-button text circle @click="goToThread">
            <el-icon><MoreFilled /></el-icon>
          </el-button>
        </div>
      </header>

      <!-- 消息列表 -->
      <div class="messages">
        <div
          v-for="message in messages"
          :key="message.id"
          class="message"
        >
          <div
            class="message-avatar"
            :style="{ background: message.avatarColor }"
          >
            {{ message.avatar }}
          </div>
          <div class="message-body">
            <div class="message-header">
              <span class="message-author">{{ message.author }}</span>
              <span class="message-time">{{ message.time }}</span>
            </div>
            <div class="message-content">{{ message.content }}</div>
            <div v-if="message.reactions.length > 0" class="message-reactions">
              <span
                v-for="(reaction, idx) in message.reactions"
                :key="idx"
                class="reaction"
              >
                {{ reaction.emoji }} {{ reaction.count }}
              </span>
            </div>
            <div
              v-if="message.threadCount"
              class="message-thread"
              @click="goToThread"
            >
              {{ message.threadCount }} {{ t('chat.replies') }}
              <el-icon><ArrowRight /></el-icon>
            </div>
          </div>
        </div>
      </div>

      <!-- 输入框 -->
      <div class="input-area">
        <div class="input-wrapper">
          <div class="input-tools">
            <el-button text circle size="small">
              <el-icon><Plus /></el-icon>
            </el-button>
            <el-button text circle size="small">
              <el-icon><Microphone /></el-icon>
            </el-button>
          </div>
          <el-input
            v-model="messageInput"
            type="textarea"
            :rows="1"
            :placeholder="t('chat.messagePlaceholder', { channel: activeChannel })"
            resize="none"
            @keydown.enter.prevent="sendMessage"
          />
          <div class="input-tools">
            <el-button
              type="primary"
              circle
              size="small"
              :disabled="!messageInput.trim()"
              @click="sendMessage"
            >
              <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </main>

    <!-- 右侧面板 -->
    <aside class="right-panel">
      <div class="panel-header">{{ t('chat.members') }}</div>
      <div class="member-list">
        <div
          v-for="group in memberGroups"
          :key="group.label"
          class="member-group"
        >
          <div class="member-group-label">{{ group.label }} — {{ group.members.length }}</div>
          <div
            v-for="member in group.members"
            :key="member.name"
            class="member-item"
          >
            <div
              class="member-dot"
              :style="{
                background: member.status === 'online' ? 'var(--wave-accent-green)' : 'var(--wave-muted)',
              }"
            />
            <div
              class="member-avatar-mini"
              :style="{ background: member.color }"
            >
              {{ member.name.charAt(0).toUpperCase() }}
            </div>
            <span class="member-name">{{ member.name }}</span>
          </div>
        </div>
      </div>
    </aside>
  </div>
</template>

<style scoped lang="scss">
.app-layout {
  display: flex;
  height: 100vh;
  background: var(--wave-bg);
  color: var(--wave-fg);
  overflow: hidden;
}

// 侧边栏
.sidebar {
  width: var(--wave-sidebar-w);
  min-width: var(--wave-sidebar-w);
  background: var(--wave-sidebar-bg);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--wave-border);
}

.sidebar-header {
  height: var(--wave-header-h);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  border-bottom: 1px solid var(--wave-border);
  font-family: var(--wave-font-display);
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;

  &:hover {
    background: var(--wave-message-hover);
  }

  .el-icon {
    color: var(--wave-muted);
  }
}

.sidebar-search {
  padding: 12px;

  :deep(.el-input__wrapper) {
    background-color: var(--wave-bg);
  }
}

.channels {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px;
}

.channel-category {
  margin-bottom: 8px;
}

.category-header {
  padding: 16px 8px 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--wave-muted);
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;

  &:hover {
    color: var(--wave-fg);
  }
}

.add-icon {
  font-size: 12px;
}

.channel {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: var(--wave-radius);
  font-size: 15px;
  color: var(--wave-muted);
  cursor: pointer;
  transition: background 0.1s;

  &:hover {
    background: var(--wave-message-hover);
    color: var(--wave-fg);
  }

  &.active {
    background: var(--wave-accent-soft);
    color: var(--wave-fg);

    .channel-prefix {
      color: var(--wave-accent);
    }
  }
}

.channel-prefix {
  color: var(--wave-muted);
  opacity: 0.6;
  font-weight: 300;
  font-size: 16px;
  display: flex;
  align-items: center;

  .el-icon {
    font-size: 14px;
  }
}

.channel-name {
  flex: 1;
}

.channel-badge {
  background: var(--wave-accent);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: var(--wave-radius-full);
  min-width: 20px;
  text-align: center;
}

// 用户信息
.user-section {
  border-top: 1px solid var(--wave-border);
  padding: 10px 12px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;

  &:hover {
    background: var(--wave-message-hover);
  }
}

.user-avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--wave-accent), var(--wave-accent-pink));
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  position: relative;
  flex-shrink: 0;
}

.status-dot {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--wave-accent-green);
  border: 2px solid var(--wave-sidebar-bg);
}

.user-info {
  flex: 1;
  min-width: 0;
}

.user-name {
  font-size: 14px;
  font-weight: 600;
}

.user-status {
  font-size: 12px;
  color: var(--wave-muted);
}

.user-controls {
  display: flex;
  gap: 4px;

  .el-button {
    color: var(--wave-muted);

    &:hover {
      color: var(--wave-fg);
    }
  }
}

// 主内容区
.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.chat-header {
  height: var(--wave-header-h);
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid var(--wave-border);
  gap: 12px;
}

.channel-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.channel-hash {
  color: var(--wave-muted);
  font-weight: 300;
  font-size: 20px;
}

.channel-title {
  font-size: 16px;
  font-weight: 600;
}

.channel-topic {
  font-size: 13px;
  color: var(--wave-muted);
}

.chat-header-right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 12px;
}

.member-avatars {
  display: flex;
}

.mini-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid var(--wave-bg);
  margin-left: -6px;
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 600;
  color: #fff;

  &:first-child {
    margin-left: 0;
  }
}

.member-count {
  font-size: 13px;
  color: var(--wave-muted);
}

// 消息列表
.messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.message {
  display: flex;
  gap: 12px;
  padding: 6px 12px;
  border-radius: var(--wave-radius);
  transition: background 0.1s;

  &:hover {
    background: var(--wave-message-hover);
  }
}

.message-avatar {
  width: 40px;
  height: 40px;
  min-width: 40px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  margin-top: 2px;
}

.message-body {
  flex: 1;
  min-width: 0;
}

.message-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 2px;
}

.message-author {
  font-size: 15px;
  font-weight: 600;

  &:hover {
    text-decoration: underline;
    cursor: pointer;
  }
}

.message-time {
  font-size: 11px;
  color: var(--wave-muted);
}

.message-content {
  font-size: 15px;
  line-height: 1.5;
}

.message-reactions {
  display: flex;
  gap: 4px;
  margin-top: 4px;
}

.reaction {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 7px;
  border-radius: var(--wave-radius-full);
  background: var(--wave-accent-soft);
  font-size: 13px;
  cursor: pointer;
  border: 1px solid transparent;

  &:hover {
    border-color: var(--wave-accent);
  }
}

.message-thread {
  margin-top: 6px;
  font-size: 13px;
  color: var(--wave-accent);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;

  &:hover {
    text-decoration: underline;
  }
}

// 输入框
.input-area {
  padding: 0 20px 20px;
  margin-top: auto;
}

.input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  background: var(--wave-surface);
  border: 1px solid var(--wave-border);
  border-radius: var(--wave-radius);
  padding: 8px 12px;
  transition: border-color 0.15s;

  &:focus-within {
    border-color: var(--wave-accent);
  }

  :deep(.el-textarea__inner) {
    background: none;
    border: none;
    color: var(--wave-fg);
    padding: 8px 4px;
    resize: none;
    max-height: 144px;
    line-height: 1.4;
    box-shadow: none;

    &::placeholder {
      color: var(--wave-muted);
    }

    &:focus {
      box-shadow: none;
    }
  }
}

.input-tools {
  display: flex;
  align-items: center;
  gap: 4px;

  .el-button {
    color: var(--wave-muted);

    &:hover {
      color: var(--wave-fg);
      background: var(--wave-message-hover);
    }

    &.is-disabled {
      color: var(--wave-muted);
    }
  }
}

// 右侧面板
.right-panel {
  width: 280px;
  min-width: 280px;
  background: var(--wave-sidebar-bg);
  border-left: 1px solid var(--wave-border);
  display: flex;
  flex-direction: column;

  @media (max-width: 860px) {
    display: none;
  }
}

.panel-header {
  height: var(--wave-header-h);
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--wave-border);
  font-weight: 600;
  font-size: 14px;
}

.member-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.member-group {
  margin-bottom: 16px;
}

.member-group-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--wave-muted);
  padding: 8px 8px 4px;
}

.member-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-radius: var(--wave-radius);
  cursor: pointer;

  &:hover {
    background: var(--wave-message-hover);
  }
}

.member-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.member-avatar-mini {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
}

.member-name {
  font-size: 14px;
  flex: 1;
}
</style>
