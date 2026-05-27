<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  ArrowLeft,
  Upload,
  ArrowRight,
} from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()

// 原始消息
const originalMessage = {
  author: 'alex',
  avatar: 'A',
  avatarColor: 'var(--accent)',
  time: '9:42 AM',
  channel: 'general',
  content: 'Hey team! 🎉 Just pushed the new design system updates to staging. Would love for everyone to take a look before we ship it live.',
}

// 回复列表
const replies = [
  {
    id: 1,
    author: 'jordan',
    avatar: 'J',
    avatarColor: 'var(--accent-green)',
    time: '9:48 AM',
    content: 'Looking great! I noticed the button component\'s hover state is a bit subtle — might want to bump the contrast there. Otherwise ship it! 🚀',
    reactions: [
      { emoji: '👍', count: 3 },
      { emoji: '🔥', count: 1 },
    ],
  },
  {
    id: 2,
    author: 'mira',
    avatar: 'M',
    avatarColor: 'var(--accent-pink)',
    time: '9:55 AM',
    isOP: true,
    content: 'Agreed with @jordan! Also the @alex spacing on mobile cards feels a bit tight. I left a comment in Figma.',
    reactions: [
      { emoji: '👀', count: 4 },
    ],
  },
  {
    id: 3,
    author: 'kaito',
    avatar: 'K',
    avatarColor: 'var(--accent-blue)',
    time: '10:12 AM',
    content: 'Tested on staging — looks solid on Chrome and Safari. There\'s a minor layout shift on Firefox at 1024px, but I think it\'s just a missing @supports query. I can patch that today.',
    reactions: [],
  },
  {
    id: 4,
    author: 'taylor',
    avatar: 'T',
    avatarColor: 'var(--accent-orange)',
    time: '10:28 AM',
    content: 'I can help with the Firefox fix after standup. Also — should we update the changelog before pushing or do it after?',
    reactions: [
      { emoji: '💯', count: 2 },
    ],
  },
  {
    id: 5,
    author: 'alex',
    avatar: 'A',
    avatarColor: 'var(--accent)',
    time: '10:35 AM',
    isOP: true,
    content: 'Great feedback everyone! @kaito go for it on the Firefox fix. @taylor let\'s do the changelog post-launch this time — I want to ship by EOD. I\'ll address the button hover and spacing comments.',
    reactions: [
      { emoji: '🎉', count: 6 },
      { emoji: '🔥', count: 3 },
    ],
  },
]

// 参与者
const participants = [
  { name: 'alex', color: 'var(--accent)' },
  { name: 'jordan', color: 'var(--accent-green)' },
  { name: 'mira', color: 'var(--accent-pink)' },
  { name: 'kaito', color: 'var(--accent-blue)' },
  { name: 'taylor', color: 'var(--accent-orange)' },
]

// 回复输入
const replyInput = ref('')

/**
 * 返回频道
 */
function goBack() {
  router.push('/app')
}

/**
 * 发送回复
 */
function sendReply() {
  if (!replyInput.value.trim()) return
  // TODO: 发送回复逻辑
  replyInput.value = ''
}
</script>

<template>
  <div class="thread-page">
    <!-- 返回栏 -->
    <div class="back-bar">
      <a href="#" @click.prevent="goBack">
        <el-icon><ArrowLeft /></el-icon>
        {{ t('thread.backToChannel', { channel: originalMessage.channel }) }}
      </a>
    </div>

    <div class="thread-container">
      <!-- 原始消息 -->
      <div class="thread-original">
        <div
          class="thread-avatar"
          :style="{ background: originalMessage.avatarColor }"
        >
          {{ originalMessage.avatar }}
        </div>
        <div class="thread-original-body">
          <div class="thread-original-header">
            <span class="name">{{ originalMessage.author }}</span>
            <span class="time">{{ originalMessage.time }}</span>
            <span class="channel">{{ t('thread.inChannel', { channel: originalMessage.channel }) }}</span>
          </div>
          <div class="thread-original-content">{{ originalMessage.content }}</div>
        </div>
      </div>

      <!-- 回复列表 -->
      <div class="thread-replies">
        <div
          v-for="reply in replies"
          :key="reply.id"
          class="reply"
        >
          <div
            class="reply-avatar"
            :style="{ background: reply.avatarColor }"
          >
            {{ reply.avatar }}
          </div>
          <div class="reply-body">
            <div class="reply-header">
              <span class="name">{{ reply.author }}</span>
              <span class="time">{{ reply.time }}</span>
              <span v-if="reply.isOP" class="badge">{{ t('thread.op') }}</span>
            </div>
            <div class="reply-content">{{ reply.content }}</div>
            <div class="reply-actions">
              <span>{{ t('chat.reply') }}</span>
              <span>{{ t('chat.react') }} 😊</span>
              <span>{{ t('chat.share') }}</span>
            </div>
            <div v-if="reply.reactions.length > 0" class="reply-reactions">
              <span
                v-for="(reaction, idx) in reply.reactions"
                :key="idx"
                class="reaction"
              >
                {{ reaction.emoji }} {{ reaction.count }}
              </span>
            </div>
          </div>
        </div>

        <!-- 系统消息 -->
        <div class="reply system-message">
          <div
            class="reply-avatar"
            style="background: var(--accent); width: 28px; height: 28px; min-width: 28px; font-size: 12px;"
          >
            W
          </div>
          <div class="reply-body">
            <div class="reply-header">
              <span class="name" style="font-size: 13px;">Wave Bot</span>
              <span class="time">10:36 AM</span>
            </div>
            <div class="reply-content" style="font-size: 13px; color: var(--muted);">
              Thread marked as resolved by <strong>alex</strong>
            </div>
          </div>
        </div>
      </div>

      <!-- 回复输入 -->
      <div class="thread-input-area">
        <div class="thread-input-wrapper">
          <div class="thread-input-tools">
            <el-button text circle size="small">
              <el-icon><Upload /></el-icon>
            </el-button>
            <el-button text circle size="small">
              <span style="font-size: 14px;">😊</span>
            </el-button>
          </div>
          <el-input
            v-model="replyInput"
            type="textarea"
            :rows="1"
            :placeholder="t('thread.replyPlaceholder')"
            resize="none"
            @keydown.enter.prevent="sendReply"
          />
          <div class="thread-input-tools">
            <el-button
              type="primary"
              circle
              size="small"
              :disabled="!replyInput.trim()"
              @click="sendReply"
            >
              <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 侧边信息面板 -->
    <aside class="thread-side-info">
      <div class="side-header">
        <h2>{{ t('thread.details.title') }}</h2>
        <p>Design system updates · #{{ originalMessage.channel }}</p>
      </div>
      <div class="side-messages">
        <div class="thread-stats">
          <div class="stat-row">
            <span>{{ t('thread.details.startedBy') }}</span>
            <span class="stat-value">{{ originalMessage.author }}</span>
          </div>
          <div class="stat-row">
            <span>{{ t('thread.details.participants') }}</span>
            <span class="stat-value">{{ participants.length }}</span>
          </div>
          <div class="stat-row">
            <span>{{ t('thread.details.replies') }}</span>
            <span class="stat-value">{{ replies.length }}</span>
          </div>
          <div class="stat-row">
            <span>{{ t('thread.details.lastReply') }}</span>
            <span class="stat-value">10:36 AM</span>
          </div>
        </div>
        <div class="participants-section">
          <p class="participants-title">{{ t('thread.details.participants') }}</p>
          <div class="participants-list">
            <div
              v-for="p in participants"
              :key="p.name"
              class="participant-item"
            >
              <span
                class="participant-avatar"
                :style="{ background: p.color }"
              >
                {{ p.name.charAt(0).toUpperCase() }}
              </span>
              <span class="participant-name">{{ p.name }}</span>
            </div>
          </div>
        </div>
      </div>
    </aside>
  </div>
</template>

<style scoped lang="scss">
.thread-page {
  display: flex;
  height: 100vh;
  background: var(--bg);
  color: var(--fg);
  overflow: hidden;
}

.back-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: color-mix(in oklch, var(--bg) 95%, transparent);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border);
  font-size: 14px;

  a {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    text-decoration: none;

    &:hover {
      color: var(--fg);
    }
  }
}

.thread-container {
  max-width: 800px;
  width: 100%;
  margin: 60px auto 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
}

// 原始消息
.thread-original {
  padding: 20px 20px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  gap: 12px;
}

.thread-avatar {
  width: 40px;
  height: 40px;
  min-width: 40px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
}

.thread-original-body {
  flex: 1;
  min-width: 0;
}

.thread-original-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;

  .name {
    font-weight: 600;
  }

  .time {
    font-size: 12px;
    color: var(--muted);
  }

  .channel {
    font-size: 12px;
    color: var(--accent);
  }
}

.thread-original-content {
  font-size: 15px;
  line-height: 1.5;
}

// 回复列表
.thread-replies {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.reply {
  display: flex;
  gap: 12px;
  padding: 8px 20px;
  transition: background 0.1s;

  &:hover {
    background: var(--message-hover);
  }

  &.system-message {
    opacity: 0.6;
  }
}

.reply-avatar {
  width: 36px;
  height: 36px;
  min-width: 36px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  margin-top: 2px;
}

.reply-body {
  flex: 1;
  min-width: 0;
}

.reply-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 2px;

  .name {
    font-size: 14px;
    font-weight: 600;
  }

  .time {
    font-size: 11px;
    color: var(--muted);
  }

  .badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent-soft);
    color: var(--accent);
  }
}

.reply-content {
  font-size: 14px;
  line-height: 1.5;
}

.reply-actions {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 12px;
  color: var(--muted);

  span {
    cursor: pointer;

    &:hover {
      color: var(--fg);
    }
  }
}

.reply-reactions {
  display: flex;
  gap: 3px;
  margin-top: 4px;

  .reaction {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: var(--radius-full);
    background: var(--accent-soft);
    font-size: 12px;
    cursor: pointer;
    border: 1px solid transparent;

    &:hover {
      border-color: var(--accent);
    }
  }
}

// 回复输入
.thread-input-area {
  padding: 12px 20px 20px;
  border-top: 1px solid var(--border);
}

.thread-input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 8px 12px;
  transition: border-color 0.15s;

  &:focus-within {
    border-color: var(--accent);
  }

  :deep(.el-textarea__inner) {
    background: none;
    border: none;
    color: var(--fg);
    padding: 8px 4px;
    resize: none;
    max-height: 120px;
    line-height: 1.4;
    box-shadow: none;

    &::placeholder {
      color: var(--muted);
    }

    &:focus {
      box-shadow: none;
    }
  }
}

.thread-input-tools {
  display: flex;
  align-items: center;
  gap: 4px;

  .el-button {
    color: var(--muted);

    &:hover {
      color: var(--fg);
      background: var(--message-hover);
    }
  }
}

// 侧边信息面板
.thread-side-info {
  width: 320px;
  min-width: 320px;
  background: var(--sidebar-bg);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;

  @media (max-width: 900px) {
    display: none;
  }
}

.side-header {
  padding: 20px;
  border-bottom: 1px solid var(--border);

  h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 4px;
  }

  p {
    font-size: 13px;
    color: var(--muted);
    margin: 0;
  }
}

.side-messages {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.thread-stats {
  font-size: 13px;
  color: var(--muted);
  margin-bottom: 16px;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;

  .stat-value {
    color: var(--fg);
  }
}

.participants-section {
  border-top: 1px solid var(--border);
  padding-top: 12px;
}

.participants-title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 8px;
}

.participants-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.participant-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.participant-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-size: 11px;
  color: #fff;
  font-weight: 600;
}

.participant-name {
  font-size: 14px;
}

@media (max-width: 640px) {
  .thread-container {
    margin-top: 52px;
  }

  .back-bar {
    padding: 8px 12px;
    font-size: 13px;
  }

  .thread-original {
    padding: 16px 12px;
  }

  .reply {
    padding: 8px 12px;
  }

  .thread-input-area {
    padding: 12px;
  }
}
</style>
