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
import { Eye, User, Users, MessageCircle } from 'lucide-vue-next'
import type { PrivacySettings, VisibilityOption } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 隐私设置数据 */
  modelValue: PrivacySettings
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
  (e: 'update:modelValue', value: PrivacySettings): void
  /** 保存设置 */
  (e: 'save', value: PrivacySettings): void
}

const emit = defineEmits<Emits>()

/**
 * 本地设置状态
 */
const localSettings = ref<PrivacySettings>({ ...props.modelValue })

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
function updateSetting<K extends keyof PrivacySettings>(
  key: K,
  value: PrivacySettings[K]
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
 * 可见性选项
 */
const visibilityOptions = [
  { label: '所有人', value: 'everyone' },
  { label: '仅好友', value: 'friends' },
  { label: '不可见', value: 'none' },
]

/**
 * 隐私设置项配置
 */
const privacyItems = [
  {
    key: 'allowStrangerMessage' as const,
    title: '允许陌生人私信',
    description: '非好友用户是否可以给你发送私信',
    icon: MessageCircle,
  },
  {
    key: 'allowRoomInvitation' as const,
    title: '允许房间邀请',
    description: '其他用户是否可以邀请你加入房间',
    icon: Users,
  },
]
</script>

<template>
  <NCard title="隐私设置" class="settings-card">
    <NSpace vertical size="large">
      <!-- 可见性设置 -->
      <div class="settings-section">
        <h4 class="section-title">可见性</h4>
        <NList bordered>
          <NListItem>
            <NThing title="在线状态可见性" description="谁可以看到你的在线状态">
              <template #avatar>
                <div class="setting-icon">
                  <User :size="18" />
                </div>
              </template>
              <template #action>
                <NSelect
                  :value="localSettings.onlineStatusVisibility"
                  :options="visibilityOptions"
                  size="small"
                  style="width: 120px"
                  @update:value="(v) => updateSetting('onlineStatusVisibility', v as VisibilityOption)"
                />
              </template>
            </NThing>
          </NListItem>
          <NListItem>
            <NThing title="个人资料可见性" description="谁可以查看你的详细资料">
              <template #avatar>
                <div class="setting-icon">
                  <Eye :size="18" />
                </div>
              </template>
              <template #action>
                <NSelect
                  :value="localSettings.profileVisibility"
                  :options="visibilityOptions"
                  size="small"
                  style="width: 120px"
                  @update:value="(v) => updateSetting('profileVisibility', v as VisibilityOption)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <NDivider />

      <!-- 互动权限 -->
      <div class="settings-section">
        <h4 class="section-title">互动权限</h4>
        <NList bordered>
          <NListItem v-for="item in privacyItems" :key="item.key">
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
          保存隐私设置
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
