<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useMessage } from 'naive-ui'
import { MessageSquare } from 'lucide-vue-next'
import { storeToRefs } from 'pinia'
import { DockBar } from '@/components/common'
import { useStatusBar } from '@/composables'
import { useRoomStore } from '@/stores'
import { useLayoutStore } from '@/store/layout'

const message = useMessage()
const route = useRoute()
const { setContent } = useStatusBar()
const roomStore = useRoomStore()
const layoutStore = useLayoutStore()

// 从 store 获取响应式状态
const { isMobile } = storeToRefs(layoutStore)

/**
 * 计算侧边栏宽度
 * 移动端下侧边栏为浮层，DockBar 应相对于整个视口居中
 */
const sidebarWidth = computed(() => isMobile.value ? '0px' : '240px')

// ==================== 状态栏 ====================

/**
 * 更新状态栏
 */
const updateStatusBar = () => {
  setContent([
    h(MessageSquare, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${roomStore.total} 个房间`,
  ])
}

// ==================== DockBar 配置 ====================

const dockConfig = ref({
  enabled: true,
  position: 'bottom' as const,
  offset: 24,
  items: [
    {
      key: 'list',
      label: '房间列表',
      icon: 'List',
      path: '/rooms/list',
    },
    {
      key: 'messages',
      label: '消息管理',
      icon: 'Mail',
      path: '/rooms/:id/messages',
      requiresParams: true,
      missingParamsMessage: '请先选择一个房间',
    },
    {
      key: 'analytics',
      label: '数据分析',
      icon: 'BarChart3',
      path: '/rooms/:id/analytics',
      requiresParams: true,
      missingParamsMessage: '请先选择一个房间',
    },
  ],
})

/**
 * 处理 DockBar 缺少参数事件
 */
const handleMissingParams = (msg: string) => {
  message.warning(msg)
}

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await roomStore.fetchRoomList({ page: 1, pageSize: 10 })
  if (success) updateStatusBar()

  // 如果当前路由有房间ID，加载房间详情
  const roomId = route.params.id as string
  if (roomId) {
    await roomStore.fetchRoomDetail(roomId)
    // 同时加载成员列表
    await roomStore.fetchMembers(roomId)
  }
})
</script>

<template>
  <div class="room-management-view">
    <!-- 子页面渲染 -->
    <router-view v-slot="{ Component }">
      <keep-alive>
        <component :is="Component" />
      </keep-alive>
    </router-view>

    <!-- DockBar 导航 -->
    <DockBar
      :config="dockConfig"
      :sidebar-width="sidebarWidth"
      @missing-params="handleMissingParams"
    />
  </div>
</template>

<style scoped>
.room-management-view {
  padding: 24px;
  min-height: 100%;
  position: relative;
}

/* 移动端适配 */
@media screen and (max-width: 768px) {
  .room-management-view {
    padding: 16px;
  }
}
</style>
