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
import { Bell, MessageCircle, AtSign, Users, Volume2, Monitor } from 'lucide-vue-next'
import type { NotificationSettings } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 通知设置数据 */
  modelValue: NotificationSettings
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
  (e: 'update:modelValue', value: NotificationSettings): void
  /** 保存设置 */
  (e: 'save', value: NotificationSettings): void
}

const emit = defineEmits<Emits>()

/**
 * 本地设置状态
 */
const localSettings = ref<NotificationSettings>({ ...props.modelValue })

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
function updateSetting<K extends keyof NotificationSettings>(
  key: K,
  value: NotificationSettings[K]
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
 * 通知类型配置
 */
const notificationTypes = [
  {
    key: 'enableDirectMessage' as const,
    title: '私信通知',
    description: '收到私信时发送通知',
    icon: MessageCircle,
  },
  {
    key: 'enableMention' as const,
    title: '@提及通知',
    description: '有人在消息中@你时发送通知',
    icon: AtSign,
  },
  {
    key: 'enableRoomInvitation' as const,
    title: '房间邀请通知',
    description: '收到房间邀请时发送通知',
    icon: Users,
  },
  {
    key: 'enableSystemNotification' as const,
    title: '系统通知',
    description: '系统公告和重要更新通知',
    icon: Bell,
  },
]

/**
 * 通知方式配置
 */
const notificationMethods = [
  {
    key: 'enableSound' as const,
    title: '声音提醒',
    description: '收到通知时播放提示音',
    icon: Volume2,
  },
  {
    key: 'enableDesktopNotification' as const,
    title: '桌面通知',
    description: '在桌面显示通知弹窗',
    icon: Monitor,
  },
]
</script>

<template>
  <NCard title="通知设置" class="settings-card">
    <NSpace vertical size="large">
      <!-- 总开关 -->
      <NList bordered>
        <NListItem>
          <NThing title="启用通知" description="接收所有类型的通知">
            <template #avatar>
              <div class="setting-icon primary">
                <Bell :size="20" />
              </div>
            </template>
            <template #action>
              <NSwitch
                :value="localSettings.enableNotification"
                @update:value="(v) => updateSetting('enableNotification', v)"
              />
            </template>
          </NThing>
        </NListItem>
      </NList>

      <NDivider />

      <!-- 通知类型 -->
      <div class="settings-section">
        <h4 class="section-title">通知类型</h4>
        <NList bordered>
          <NListItem v-for="item in notificationTypes" :key="item.key">
            <NThing :title="item.title" :description="item.description">
              <template #avatar>
                <div class="setting-icon">
                  <component :is="item.icon" :size="18" />
                </div>
              </template>
              <template #action>
                <NSwitch
                  :value="localSettings[item.key]"
                  :disabled="!localSettings.enableNotification"
                  @update:value="(v) => updateSetting(item.key, v)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <NDivider />

      <!-- 通知方式 -->
      <div class="settings-section">
        <h4 class="section-title">通知方式</h4>
        <NList bordered>
          <NListItem v-for="item in notificationMethods" :key="item.key">
            <NThing :title="item.title" :description="item.description">
              <template #avatar>
                <div class="setting-icon">
                  <component :is="item.icon" :size="18" />
                </div>
              </template>
              <template #action>
                <NSwitch
                  :value="localSettings[item.key]"
                  :disabled="!localSettings.enableNotification"
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
          保存通知设置
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
