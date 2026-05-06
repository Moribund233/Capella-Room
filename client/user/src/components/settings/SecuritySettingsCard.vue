<script setup lang="ts">
import { ref, watch } from 'vue'
import { useDialog } from 'naive-ui'
import {
  NCard,
  NSpace,
  NSwitch,
  NList,
  NListItem,
  NThing,
  NButton,
  NDivider,
  NTag,
  NEmpty,
  NSkeleton,
} from 'naive-ui'
import { Shield, AlertTriangle, Smartphone, Laptop, Tablet, Monitor, LogOut, Ban } from 'lucide-vue-next'
import type { SecuritySettings, LoginDevice, DeviceType } from '@/types/settings'
import type { Component } from 'vue'

/**
 * 组件属性定义
 */
interface Props {
  /** 安全设置数据 */
  modelValue: SecuritySettings
  /** 登录设备列表 */
  devices: LoginDevice[]
  /** 加载中状态 */
  loading?: boolean
  /** 设备操作加载中 */
  deviceLoading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  deviceLoading: false,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新设置 */
  (e: 'update:modelValue', value: SecuritySettings): void
  /** 保存设置 */
  (e: 'save', value: SecuritySettings): void
  /** 登出设备 */
  (e: 'logoutDevice', deviceId: string): void
  /** 禁用设备 */
  (e: 'blockDevice', deviceId: string): void
  /** 启用设备 */
  (e: 'unblockDevice', deviceId: string): void
}

const emit = defineEmits<Emits>()

const dialog = useDialog()

/**
 * 本地设置状态
 */
const localSettings = ref<SecuritySettings>({ ...props.modelValue })

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
function updateSetting<K extends keyof SecuritySettings>(
  key: K,
  value: SecuritySettings[K]
) {
  localSettings.value = { ...localSettings.value, [key]: value }
  emit('update:modelValue', localSettings.value)
  emit('save', localSettings.value)
}

/**
 * 设备图标映射
 */
const deviceIconMap: Record<DeviceType, Component> = {
  mobile: Smartphone,
  tablet: Tablet,
  desktop: Laptop,
  unknown: Monitor,
}

/**
 * 获取设备类型标签
 */
function getDeviceTypeLabel(type: DeviceType): string {
  const labels: Record<DeviceType, string> = {
    mobile: '手机',
    tablet: '平板',
    desktop: '电脑',
    unknown: '未知设备',
  }
  return labels[type]
}

/**
 * 格式化日期
 */
function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleString('zh-CN')
}

/**
 * 确认登出设备
 */
function confirmLogout(device: LoginDevice) {
  dialog.warning({
    title: '确认登出',
    content: `确定要登出设备 "${device.deviceName}" 吗？`,
    positiveText: '确认',
    negativeText: '取消',
    onPositiveClick: () => {
      emit('logoutDevice', device.deviceId)
    },
  })
}

/**
 * 确认禁用设备
 */
function confirmBlock(device: LoginDevice) {
  dialog.error({
    title: '确认禁用',
    content: `禁用后，该设备将无法再登录你的账号。确定要禁用 "${device.deviceName}" 吗？`,
    positiveText: '确认禁用',
    negativeText: '取消',
    onPositiveClick: () => {
      emit('blockDevice', device.deviceId)
    },
  })
}

/**
 * 启用设备
 */
function handleUnblock(device: LoginDevice) {
  emit('unblockDevice', device.deviceId)
}
</script>

<template>
  <NCard title="账号安全" class="settings-card">
    <NSpace vertical size="large">
      <!-- 安全设置 -->
      <NList bordered>
        <NListItem>
          <NThing
            title="异地登录提醒"
            description="检测到异常登录位置时发送提醒通知"
          >
            <template #avatar>
              <div class="setting-icon warning">
                <AlertTriangle :size="20" />
              </div>
            </template>
            <template #action>
              <NSwitch
                :value="localSettings.enableAbnormalLoginAlert"
                @update:value="(v) => updateSetting('enableAbnormalLoginAlert', v)"
              />
            </template>
          </NThing>
        </NListItem>
        <NListItem>
          <NThing
            title="单设备登录"
            description="只允许一个设备同时在线，新设备登录会自动登出其他设备"
          >
            <template #avatar>
              <div class="setting-icon primary">
                <Shield :size="20" />
              </div>
            </template>
            <template #action>
              <NSwitch
                :value="localSettings.enableSingleDeviceLogin"
                @update:value="(v) => updateSetting('enableSingleDeviceLogin', v)"
              />
            </template>
          </NThing>
        </NListItem>
      </NList>

      <NDivider />

      <!-- 登录设备管理 -->
      <div class="settings-section">
        <h4 class="section-title">登录设备管理</h4>

        <NSkeleton v-if="loading" text :repeat="3" />

        <NEmpty v-else-if="devices.length === 0" description="暂无登录设备" />

        <NList v-else bordered>
          <NListItem v-for="device in devices" :key="device.deviceId">
            <NThing
              :title="device.deviceName"
              :description="`${getDeviceTypeLabel(device.deviceType)} · ${device.location} · ${formatDate(device.lastActiveAt)}`"
            >
              <template #avatar>
                <div class="setting-icon" :class="{ disabled: device.isBlocked }">
                  <component :is="deviceIconMap[device.deviceType]" :size="18" />
                </div>
              </template>
              <template #header-extra>
                <NTag v-if="device.isCurrent" type="success" size="small">当前设备</NTag>
                <NTag v-if="device.isBlocked" type="error" size="small">已禁用</NTag>
              </template>
              <template #action>
                <NSpace>
                  <NButton
                    v-if="!device.isCurrent && !device.isBlocked"
                    size="small"
                    @click="confirmLogout(device)"
                    :loading="deviceLoading"
                  >
                    <template #icon>
                      <LogOut :size="14" />
                    </template>
                    登出
                  </NButton>
                  <NButton
                    v-if="!device.isCurrent && !device.isBlocked"
                    size="small"
                    type="error"
                    ghost
                    @click="confirmBlock(device)"
                    :loading="deviceLoading"
                  >
                    <template #icon>
                      <Ban :size="14" />
                    </template>
                    禁用
                  </NButton>
                  <NButton
                    v-if="device.isBlocked"
                    size="small"
                    type="primary"
                    @click="handleUnblock(device)"
                    :loading="deviceLoading"
                  >
                    启用
                  </NButton>
                </NSpace>
              </template>
            </NThing>
          </NListItem>
        </NList>
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

.setting-icon.warning {
  background: var(--color-warning-soft, rgba(255, 152, 0, 0.1));
  color: var(--color-warning, #ff9800);
}

.setting-icon.disabled {
  background: var(--color-background-mute);
  color: var(--color-text-disabled);
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
</style>
