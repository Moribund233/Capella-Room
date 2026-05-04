<template>
  <n-card title="系统动态" :bordered="false" class="timeline-card">
    <n-timeline>
      <n-timeline-item
        v-for="item in timelineItems"
        :key="item.id"
        :type="item.type"
        :title="item.title"
        :content="item.content"
        :time="item.time"
      >
        <template #icon>
          <n-icon :component="item.icon" />
        </template>
      </n-timeline-item>
    </n-timeline>
  </n-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NCard, NTimeline, NTimelineItem, NIcon } from 'naive-ui'
import {
  CheckCircle2,
  AlertCircle,
  MessageSquare,
  UserPlus,
  Settings,
  type LucideIcon,
} from 'lucide-vue-next'
import type { TimelineItemProps } from 'naive-ui'

interface TimelineItem {
  id: string
  type: TimelineItemProps['type']
  title: string
  content: string
  time: string
  icon: LucideIcon
}

const timelineItems = ref<TimelineItem[]>([
  {
    id: '1',
    type: 'success',
    title: '系统更新完成',
    content: '系统已成功更新至 v1.2.0 版本，新增深色主题支持',
    time: '10分钟前',
    icon: CheckCircle2,
  },
  {
    id: '2',
    type: 'info',
    title: '新用户加入',
    content: '用户 "张三" 加入了技术交流群',
    time: '30分钟前',
    icon: UserPlus,
  },
  {
    id: '3',
    type: 'default',
    title: '新消息提醒',
    content: 'Vue3 开发者社区有新消息：关于 Composition API 的讨论',
    time: '1小时前',
    icon: MessageSquare,
  },
  {
    id: '4',
    type: 'warning',
    title: '系统维护通知',
    content: '计划于今晚 02:00 进行系统维护，预计持续 30 分钟',
    time: '2小时前',
    icon: AlertCircle,
  },
  {
    id: '5',
    type: 'default',
    title: '功能更新',
    content: '新增房间搜索功能，支持按关键词查找房间',
    time: '昨天',
    icon: Settings,
  },
])
</script>

<style scoped>
.timeline-card {
  height: 100%;
}
</style>
