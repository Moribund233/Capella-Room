<script setup lang="ts">
import { computed } from 'vue'
import {
  Inbox,
  Search,
  MessageSquare,
  Users,
  FileQuestion,
  WifiOff,
  type LucideIcon,
} from 'lucide-vue-next'

type EmptyType = 'default' | 'search' | 'message' | 'user' | 'data' | 'offline'

/**
 * 空状态组件
 * 用于页面无数据时的友好提示
 */

const props = withDefaults(
  defineProps<{
    /** 空状态类型 */
    type?: EmptyType
    /** 自定义标题 */
    title?: string
    /** 自定义描述 */
    description?: string
    /** 是否显示操作按钮 */
    showAction?: boolean
    /** 操作按钮文字 */
    actionText?: string
  }>(),
  {
    type: 'default',
    title: '',
    description: '',
    showAction: false,
    actionText: '刷新',
  }
)

const emit = defineEmits<{
  (e: 'action'): void
}>()

const iconMap: Record<EmptyType, LucideIcon> = {
  default: Inbox,
  search: Search,
  message: MessageSquare,
  user: Users,
  data: FileQuestion,
  offline: WifiOff,
}

const defaultTitles: Record<EmptyType, string> = {
  default: '暂无数据',
  search: '未找到结果',
  message: '暂无消息',
  user: '暂无用户',
  data: '数据为空',
  offline: '网络已断开',
}

const defaultDescriptions: Record<EmptyType, string> = {
  default: '这里还没有任何内容',
  search: '请尝试其他关键词搜索',
  message: '开始发送第一条消息吧',
  user: '还没有用户加入',
  data: '暂时没有相关数据',
  offline: '请检查网络连接后重试',
}

const displayIcon = computed(() => iconMap[props.type])
const displayTitle = computed(() => props.title || defaultTitles[props.type])
const displayDescription = computed(() => props.description || defaultDescriptions[props.type])
</script>

<template>
  <div class="empty-state" role="status" aria-live="polite">
    <div class="empty-state__icon">
      <component :is="displayIcon" :size="64" />
    </div>

    <h3 class="empty-state__title">
      {{ displayTitle }}
    </h3>

    <p class="empty-state__description">
      {{ displayDescription }}
    </p>

    <button
      v-if="showAction"
      class="empty-state__action"
      @click="emit('action')"
    >
      {{ actionText }}
    </button>
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
}

.empty-state__icon {
  color: var(--empty-icon-color, rgba(0, 0, 0, 0.25));
  margin-bottom: 16px;
}

.empty-state__title {
  font-size: 16px;
  font-weight: 500;
  color: var(--text-primary, rgba(0, 0, 0, 0.88));
  margin: 0 0 8px;
}

.empty-state__description {
  font-size: 14px;
  color: var(--text-secondary, rgba(0, 0, 0, 0.45));
  margin: 0 0 24px;
  line-height: 1.5;
}

.empty-state__action {
  padding: 8px 20px;
  font-size: 14px;
  font-weight: 500;
  color: var(--primary-color, #1890ff);
  background: transparent;
  border: 1px solid var(--primary-color, #1890ff);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.empty-state__action:hover {
  color: white;
  background: var(--primary-color, #1890ff);
}
</style>
