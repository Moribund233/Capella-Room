<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NModal,
  NList,
  NListItem,
  NThing,
  NTag,
  NEmpty,
  NSkeleton,
  NPagination,
} from 'naive-ui'
import { Smartphone, Tablet, Monitor, Laptop, AlertTriangle } from 'lucide-vue-next'
import { useSettingsStore } from '@/stores/settings'
import type { Component } from 'vue'
import type { LoginHistory } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 是否显示 */
  visible: boolean
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 关闭弹窗 */
  (e: 'update:visible', value: boolean): void
}

const emit = defineEmits<Emits>()

const settingsStore = useSettingsStore()

/** 登录历史列表 */
const loginHistory = ref<LoginHistory[]>([])
/** 加载状态 */
const loading = ref(false)
/** 总记录数 */
const total = ref(0)
/** 当前页码 */
const currentPage = ref(1)
/** 每页数量 */
const pageSize = ref(10)

/**
 * 设备图标映射
 */
const deviceIconMap: Record<string, Component> = {
  mobile: Smartphone,
  tablet: Tablet,
  desktop: Laptop,
  unknown: Monitor,
}

/**
 * 加载登录历史
 */
async function loadLoginHistory() {
  loading.value = true

  try {
    const offset = (currentPage.value - 1) * pageSize.value
    const success = await settingsStore.loadLoginHistory({
      limit: pageSize.value,
      offset,
    })

    if (success) {
      loginHistory.value = settingsStore.loginHistory
      // 假设 store 中有 total 字段，这里使用列表长度作为临时方案
      total.value = loginHistory.value.length + (loginHistory.value.length >= pageSize.value ? pageSize.value : 0)
    }
  } finally {
    loading.value = false
  }
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateStr: string): string {
  return new Date(dateStr).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * 获取设备类型标签
 */
function getDeviceTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    mobile: '手机',
    tablet: '平板',
    desktop: '电脑',
    unknown: '未知设备',
  }
  return labels[type] || '未知设备'
}

/**
 * 获取登录结果标签类型
 */
function getResultTagType(result: string): 'success' | 'error' {
  return result === 'success' ? 'success' : 'error'
}

/**
 * 获取登录结果文本
 */
function getResultText(result: string): string {
  return result === 'success' ? '成功' : '失败'
}

/**
 * 获取风险等级标签类型
 */
function getRiskTagType(risk: string): 'default' | 'warning' | 'error' {
  const map: Record<string, 'default' | 'warning' | 'error'> = {
    low: 'default',
    medium: 'warning',
    high: 'error',
  }
  return map[risk] || 'default'
}

/**
 * 获取风险等级文本
 */
function getRiskText(risk: string): string {
  const map: Record<string, string> = {
    low: '低风险',
    medium: '中风险',
    high: '高风险',
  }
  return map[risk] || '未知'
}

/**
 * 处理页码变化
 */
function handlePageChange(page: number) {
  currentPage.value = page
  loadLoginHistory()
}

/**
 * 处理关闭
 */
function handleClose() {
  emit('update:visible', false)
}

// 监听 visible 变化
watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      currentPage.value = 1
      loadLoginHistory()
    }
  }
)
</script>

<template>
  <NModal
    :show="visible"
    preset="card"
    title="登录历史"
    style="width: 90%; max-width: 600px; max-height: 80vh"
    @close="handleClose"
    @mask-click="handleClose"
  >
    <div class="login-history-modal">
      <!-- 加载中 -->
      <div v-if="loading" class="login-history-modal__loading">
        <NSkeleton text :repeat="5" />
      </div>

      <!-- 空状态 -->
      <NEmpty
        v-else-if="loginHistory.length === 0"
        description="暂无登录历史记录"
      />

      <!-- 历史列表 -->
      <NList v-else bordered>
        <NListItem
          v-for="record in loginHistory"
          :key="record.id"
        >
          <NThing>
            <template #avatar>
              <div
                class="login-history-modal__icon"
                :class="`login-history-modal__icon--${record.result}`"
              >
                <component
                  :is="deviceIconMap[record.deviceType] || Monitor"
                  :size="20"
                />
              </div>
            </template>

            <template #header>
              <div class="login-history-modal__header">
                <span class="login-history-modal__device">
                  {{ record.deviceName || getDeviceTypeLabel(record.deviceType) }}
                </span>
                <div class="login-history-modal__tags">
                  <NTag
                    :type="getResultTagType(record.result)"
                    size="small"
                  >
                    {{ getResultText(record.result) }}
                  </NTag>
                  <NTag
                    v-if="record.riskLevel !== 'low'"
                    :type="getRiskTagType(record.riskLevel)"
                    size="small"
                  >
                    <template #icon>
                      <AlertTriangle v-if="record.riskLevel === 'high'" :size="12" />
                      <Shield v-else :size="12" />
                    </template>
                    {{ getRiskText(record.riskLevel) }}
                  </NTag>
                </div>
              </div>
            </template>

            <template #description>
              <div class="login-history-modal__details">
                <span class="login-history-modal__ip">IP: {{ record.ipAddress }}</span>
                <span class="login-history-modal__time">
                  {{ formatDateTime(record.loginAt) }}
                </span>
              </div>
            </template>
          </NThing>
        </NListItem>
      </NList>

      <!-- 分页 -->
      <div
        v-if="loginHistory.length > 0 && total > pageSize"
        class="login-history-modal__pagination"
      >
        <NPagination
          v-model:page="currentPage"
          :page-size="pageSize"
          :item-count="total"
          @update:page="handlePageChange"
        />
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.login-history-modal {
  max-height: 60vh;
  overflow-y: auto;
}

.login-history-modal__loading {
  padding: 20px;
}

.login-history-modal__icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background-soft);
  color: var(--color-text-secondary);
}

.login-history-modal__icon--success {
  background: var(--color-success-light, rgba(82, 196, 26, 0.1));
  color: var(--color-success, #52c41a);
}

.login-history-modal__icon--failed {
  background: var(--color-error-light, rgba(255, 77, 79, 0.1));
  color: var(--color-error, #ff4d4f);
}

.login-history-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.login-history-modal__device {
  font-weight: 500;
  color: var(--color-text-primary);
}

.login-history-modal__tags {
  display: flex;
  gap: 6px;
}

.login-history-modal__details {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: var(--font-size-small, 12px);
  color: var(--color-text-secondary);
}

.login-history-modal__ip {
  font-family: monospace;
}

.login-history-modal__pagination {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}
</style>
