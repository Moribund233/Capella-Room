<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NCard,
  NSpace,
  NRadioGroup,
  NRadioButton,
  NList,
  NListItem,
  NThing,
  NButton,
} from 'naive-ui'
import { Sun, Moon, Monitor, Palette } from 'lucide-vue-next'
import type { ThemeType } from '@/stores/theme'
import type { Component } from 'vue'

/**
 * 组件属性定义
 */
interface Props {
  /** 当前主题 */
  modelValue: ThemeType
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新主题 */
  (e: 'update:modelValue', value: ThemeType): void
  /** 打开个性化设置 */
  (e: 'openPersonalization'): void
}

const emit = defineEmits<Emits>()

/**
 * 本地主题状态
 */
const localTheme = ref<ThemeType>(props.modelValue)

/**
 * 监听外部数据变化
 */
watch(
  () => props.modelValue,
  (newVal) => {
    localTheme.value = newVal
  }
)

/**
 * 更新主题
 */
function updateTheme(value: ThemeType) {
  localTheme.value = value
  emit('update:modelValue', value)
}

/**
 * 主题选项
 */
const themeOptions: { label: string; value: ThemeType; icon: Component; description: string }[] = [
  { 
    label: '浅色', 
    value: 'light', 
    icon: Sun,
    description: '明亮的界面风格，适合日间使用'
  },
  { 
    label: '深色', 
    value: 'dark', 
    icon: Moon,
    description: '暗色的界面风格，适合夜间使用'
  },
  { 
    label: '跟随系统', 
    value: 'system', 
    icon: Monitor,
    description: '自动跟随系统主题设置'
  },
]
</script>

<template>
  <NCard title="主题设置" class="settings-card">
    <NSpace vertical size="large">
      <!-- 主题选择 -->
      <NRadioGroup 
        :value="localTheme" 
        @update:value="updateTheme"
        class="theme-radio-group"
      >
        <NSpace vertical>
          <NRadioButton 
            v-for="option in themeOptions" 
            :key="option.value" 
            :value="option.value"
            class="theme-radio-button"
          >
            <div class="theme-option">
              <div class="theme-icon" :class="option.value">
                <component :is="option.icon" :size="24" />
              </div>
              <div class="theme-info">
                <div class="theme-label">{{ option.label }}</div>
                <div class="theme-desc">{{ option.description }}</div>
              </div>
            </div>
          </NRadioButton>
        </NSpace>
      </NRadioGroup>

      <!-- 个性化入口 -->
      <NList bordered class="personalization-list">
        <NListItem @click="$emit('openPersonalization')" class="personalization-item">
          <NThing title="外观个性化" description="自定义背景、强调色等">
            <template #avatar>
              <div class="setting-icon primary">
                <Palette :size="20" />
              </div>
            </template>
            <template #action>
              <NButton text type="primary">去设置</NButton>
            </template>
          </NThing>
        </NListItem>
      </NList>
    </NSpace>
  </NCard>
</template>

<style scoped>
.settings-card {
  margin-bottom: 16px;
}

.theme-radio-group {
  width: 100%;
}

.theme-radio-group :deep(.n-radio-group__wrapper) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.theme-radio-button {
  width: 100%;
  height: auto;
  padding: 16px;
}

.theme-radio-button :deep(.n-radio__label) {
  width: 100%;
}

.theme-option {
  display: flex;
  align-items: center;
  gap: 16px;
  text-align: left;
}

.theme-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.theme-icon.light {
  background: #f5f5f5;
  color: #333;
}

.theme-icon.dark {
  background: #1a1a1a;
  color: #fff;
}

.theme-icon.system {
  background: linear-gradient(135deg, #f5f5f5 50%, #1a1a1a 50%);
  color: #666;
}

.theme-info {
  flex: 1;
}

.theme-label {
  font-size: 16px;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: 4px;
}

.theme-desc {
  font-size: 13px;
  color: var(--color-text-secondary);
}

.setting-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: var(--color-background-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
}

.setting-icon.primary {
  background: var(--color-primary-soft);
  color: var(--color-primary);
}

.personalization-list {
  margin-top: 8px;
}

.personalization-item {
  cursor: pointer;
}

.personalization-item:hover {
  background: var(--color-background-soft);
}
</style>
