<template>
  <div class="personalization-modal">
    <n-tabs type="line" animated v-model:value="activeTab">
      <!-- 侧边栏设置 -->
      <n-tab-pane name="sidebar" tab="侧边栏">
        <div class="settings-group">
          <h4 class="group-title">尺寸设置</h4>
          <div class="setting-item">
            <span class="setting-label">展开宽度</span>
            <n-slider v-model:value="localStyles.sidebarWidthExpanded" :min="160" :max="320" :step="8"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.sidebarWidthExpanded" :min="160" :max="320" :step="8"
              size="small" style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">折叠宽度</span>
            <n-slider v-model:value="localStyles.sidebarWidthCollapsed" :min="48" :max="80" :step="4"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.sidebarWidthCollapsed" :min="48" :max="80" :step="4" size="small"
              style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">高度比例</span>
            <n-slider v-model:value="localStyles.sidebarHeightRatio" :min="0.5" :max="1" :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`" />
            <span class="setting-value">{{ Math.round(localStyles.sidebarHeightRatio * 100) }}%</span>
          </div>

          <h4 class="group-title">外观设置</h4>
          <div class="setting-item">
            <span class="setting-label">圆角</span>
            <n-slider v-model:value="localStyles.sidebarBorderRadius" :min="0" :max="24" :step="2"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.sidebarBorderRadius" :min="0" :max="24" :step="2" size="small"
              style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">透明度</span>
            <n-slider v-model:value="localStyles.sidebarOpacity" :min="0.5" :max="1" :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`" />
            <span class="setting-value">{{ Math.round(localStyles.sidebarOpacity * 100) }}%</span>
          </div>
        </div>
      </n-tab-pane>

      <!-- 顶部设置 -->
      <n-tab-pane name="header" tab="顶部">
        <div class="settings-group">
          <h4 class="group-title">尺寸设置</h4>
          <div class="setting-item">
            <span class="setting-label">高度</span>
            <n-slider v-model:value="localStyles.headerHeight" :min="48" :max="80" :step="4"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.headerHeight" :min="48" :max="80" :step="4" size="small"
              style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">宽度比例</span>
            <n-slider v-model:value="localStyles.headerWidthRatio" :min="0.5" :max="1" :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`" />
            <span class="setting-value">{{ Math.round(localStyles.headerWidthRatio * 100) }}%</span>
          </div>

          <h4 class="group-title">外观设置</h4>
          <div class="setting-item">
            <span class="setting-label">圆角</span>
            <n-slider v-model:value="localStyles.headerBorderRadius" :min="0" :max="24" :step="2"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.headerBorderRadius" :min="0" :max="24" :step="2" size="small"
              style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">透明度</span>
            <n-slider v-model:value="localStyles.headerOpacity" :min="0.5" :max="1" :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`" />
            <span class="setting-value">{{ Math.round(localStyles.headerOpacity * 100) }}%</span>
          </div>
        </div>
      </n-tab-pane>

      <!-- 底部设置 -->
      <n-tab-pane name="footer" tab="底部">
        <div class="settings-group">
          <h4 class="group-title">尺寸设置</h4>
          <div class="setting-item">
            <span class="setting-label">高度</span>
            <n-slider v-model:value="localStyles.footerHeight" :min="32" :max="64" :step="4"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.footerHeight" :min="32" :max="64" :step="4" size="small"
              style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">宽度比例</span>
            <n-slider v-model:value="localStyles.footerWidthRatio" :min="0.5" :max="1" :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`" />
            <span class="setting-value">{{ Math.round(localStyles.footerWidthRatio * 100) }}%</span>
          </div>

          <h4 class="group-title">外观设置</h4>
          <div class="setting-item">
            <span class="setting-label">圆角</span>
            <n-slider v-model:value="localStyles.footerBorderRadius" :min="0" :max="24" :step="2"
              :format-tooltip="(v) => `${v}px`" />
            <n-input-number v-model:value="localStyles.footerBorderRadius" :min="0" :max="24" :step="2" size="small"
              style="width: 80px" />
          </div>
          <div class="setting-item">
            <span class="setting-label">透明度</span>
            <n-slider v-model:value="localStyles.footerOpacity" :min="0.5" :max="1" :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`" />
            <span class="setting-value">{{ Math.round(localStyles.footerOpacity * 100) }}%</span>
          </div>
        </div>
      </n-tab-pane>
    </n-tabs>

    <!-- 底部操作 -->
    <div class="modal-footer">
      <n-button @click="handleReset">重置默认</n-button>
      <n-button type="primary" @click="handleConfirm">应用设置</n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch, ref } from 'vue'
import { NTabs, NTabPane, NSlider, NInputNumber, NButton } from 'naive-ui'
import type { LayoutStyles } from '@/store/layout'

/**
 * 组件属性定义
 */
interface Props {
  /** 当前布局样式 */
  modelValue: LayoutStyles
  /** 默认激活的标签页 */
  defaultTab?: string
}

const props = withDefaults(defineProps<Props>(), {
  defaultTab: 'sidebar'
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新样式 */
  (e: 'update:modelValue', value: LayoutStyles): void
  /** 确认 */
  (e: 'confirm'): void
  /** 重置 */
  (e: 'reset'): void
}

const emit = defineEmits<Emits>()

/**
 * 当前激活的标签页
 */
const activeTab = ref(props.defaultTab)

/**
 * 本地样式状态
 */
const localStyles = reactive<LayoutStyles>({ ...props.modelValue })

/**
 * 监听外部样式变化
 */
watch(
  () => props.modelValue,
  (newValue) => {
    Object.assign(localStyles, newValue)
  },
  { deep: true },
)

/**
 * 监听默认标签页变化
 */
watch(
  () => props.defaultTab,
  (newValue) => {
    if (newValue) {
      activeTab.value = newValue
    }
  }
)

/**
 * 处理确认
 */
function handleConfirm(): void {
  emit('update:modelValue', { ...localStyles })
  emit('confirm')
}

/**
 * 处理重置
 */
function handleReset(): void {
  emit('reset')
}
</script>

<style scoped>
.personalization-modal {
  padding: 16px 0;
  min-height: 400px;
}

.settings-group {
  margin-bottom: 24px;
}

.group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color-split);
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.setting-label {
  width: 80px;
  font-size: 13px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.setting-item :deep(.n-slider) {
  flex: 1;
}

.setting-value {
  width: 50px;
  font-size: 13px;
  color: var(--text-primary);
  text-align: right;
  flex-shrink: 0;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color-split);
}
</style>
