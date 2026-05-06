<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  NCard,
  NSpace,
  NSwitch,
  NList,
  NListItem,
  NThing,
  NButton,
  NDivider,
} from 'naive-ui'
import { Eye, CheckCircle, Keyboard, Moon } from 'lucide-vue-next'
import type { MessageSettings } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 消息设置数据 */
  modelValue: MessageSettings
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
  (e: 'update:modelValue', value: MessageSettings): void
  /** 保存设置 */
  (e: 'save', value: MessageSettings): void
}

const emit = defineEmits<Emits>()

/**
 * 本地设置状态
 */
const localSettings = ref<MessageSettings>({ ...props.modelValue })

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
function updateSetting<K extends keyof MessageSettings>(
  key: K,
  value: MessageSettings[K]
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
 * 消息设置项配置
 */
const messageItems = [
  {
    key: 'showMessagePreview' as const,
    title: '消息预览',
    description: '在通知中显示消息内容预览',
    icon: Eye,
  },
  {
    key: 'enableReadReceipt' as const,
    title: '已读回执',
    description: '允许其他人看到你已读消息的状态',
    icon: CheckCircle,
  },
  {
    key: 'showTypingStatus' as const,
    title: '输入状态',
    description: '显示对方正在输入的提示',
    icon: Keyboard,
  },
  {
    key: 'enableDoNotDisturb' as const,
    title: '免打扰模式',
    description: '开启后，非重要通知将被静音',
    icon: Moon,
  },
]
</script>

<template>
  <NCard title="消息设置" class="settings-card">
    <NSpace vertical size="large">
      <!-- 消息设置列表 -->
      <NList bordered>
        <NListItem v-for="item in messageItems" :key="item.key">
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

      <NDivider />

      <!-- 说明 -->
      <div class="settings-hint">
        <p>这些设置将影响你在所有房间和私信中的消息体验。</p>
      </div>

      <!-- 保存按钮 -->
      <div class="card-actions">
        <NButton type="primary" :loading="saving" @click="handleSave">
          保存消息设置
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

.settings-hint {
  padding: 12px 16px;
  background: var(--color-background-soft);
  border-radius: 8px;
}

.settings-hint p {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.card-actions {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
}
</style>
