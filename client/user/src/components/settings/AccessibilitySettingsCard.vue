<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NCard,
  NSpace,
  NSwitch,
  NSelect,
  NList,
  NListItem,
  NThing,
  NButton,
  NDivider,
} from 'naive-ui'
import { Type, Eye, Zap, Monitor } from 'lucide-vue-next'
import type { AccessibilitySettings, FontSize } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 无障碍设置数据 */
  modelValue: AccessibilitySettings
  /** 保存中状态 */
  saving?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  saving: false,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新设置 */
  (e: 'update:modelValue', value: AccessibilitySettings): void
  /** 保存设置 */
  (e: 'save', value: AccessibilitySettings): void
}

const emit = defineEmits<Emits>()

/**
 * 本地设置状态
 */
const localSettings = ref<AccessibilitySettings>({ ...props.modelValue })

/**
 * 监听外部数据变化
 */
watch(
  () => props.modelValue,
  (newVal) => {
    localSettings.value = { ...newVal }
  },
  { deep: true }
)

/**
 * 更新设置项
 */
function updateSetting<K extends keyof AccessibilitySettings>(
  key: K,
  value: AccessibilitySettings[K]
) {
  localSettings.value = { ...localSettings.value, [key]: value }
  emit('update:modelValue', localSettings.value)
}

/**
 * 保存设置
 */
function handleSave() {
  emit('save', localSettings.value)
}

/**
 * 字体大小选项
 */
const fontSizeOptions: { label: string; value: FontSize }[] = [
  { label: '小', value: 'small' },
  { label: '中', value: 'medium' },
  { label: '大', value: 'large' },
]

/**
 * 无障碍设置项配置
 */
const accessibilityItems = [
  {
    key: 'reduceMotion' as const,
    title: '减少动效',
    description: '降低界面动画效果，适合对动画敏感的用户',
    icon: Zap,
  },
  {
    key: 'highContrast' as const,
    title: '高对比度',
    description: '增强界面元素的对比度，提高可读性',
    icon: Eye,
  },
  {
    key: 'compactMode' as const,
    title: '紧凑模式',
    description: '减小界面元素间距，显示更多内容',
    icon: Monitor,
  },
]
</script>

<template>
  <NCard title="无障碍设置" class="settings-card">
    <NSpace vertical size="large">
      <!-- 字体大小 -->
      <NList bordered>
        <NListItem>
          <NThing title="字体大小" description="调整应用中的文字大小">
            <template #avatar>
              <div class="setting-icon primary">
                <Type :size="20" />
              </div>
            </template>
            <template #action>
              <NSelect
                :value="localSettings.fontSize"
                :options="fontSizeOptions"
                size="small"
                style="width: 100px"
                @update:value="(v) => updateSetting('fontSize', v as FontSize)"
              />
            </template>
          </NThing>
        </NListItem>
      </NList>

      <NDivider />

      <!-- 辅助功能 -->
      <div class="settings-section">
        <h4 class="section-title">辅助功能</h4>
        <NList bordered>
          <NListItem v-for="item in accessibilityItems" :key="item.key">
            <NThing :title="item.title" :description="item.description">
              <template #avatar>
                <div class="setting-icon">
                  <component :is="item.icon" :size="18" />
                </div>
              </template>
              <template #action>
                <NSwitch
                  :value="localSettings[item.key]"
                  @update:value="(v) => updateSetting(item.key, v)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <!-- 保存按钮 -->
      <div class="card-actions">
        <NButton type="primary" :loading="saving" @click="handleSave">
          保存无障碍设置
        </NButton>
      </div>
    </NSpace>
  </NCard>
</template>

<style scoped>
.settings-card {
  margin-bottom: 16px;
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

.settings-section {
  margin-top: 8px;
}

.section-title {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.card-actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
}
</style>
