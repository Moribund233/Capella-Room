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
import { Image, Download, Play, Trash2 } from 'lucide-vue-next'
import type { MediaSettings, ImageQuality } from '@/types/settings'

/**
 * 组件属性定义
 */
interface Props {
  /** 媒体与存储设置数据 */
  modelValue: MediaSettings
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
  (e: 'update:modelValue', value: MediaSettings): void
  /** 保存设置 */
  (e: 'save', value: MediaSettings): void
  /** 清除缓存 */
  (e: 'clearCache'): void
}

const emit = defineEmits<Emits>()

/**
 * 本地设置状态
 */
const localSettings = ref<MediaSettings>({ ...props.modelValue })

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
function updateSetting<K extends keyof MediaSettings>(
  key: K,
  value: MediaSettings[K]
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
 * 图片质量选项
 */
const imageQualityOptions: { label: string; value: ImageQuality }[] = [
  { label: '原图', value: 'original' },
  { label: '高', value: 'high' },
  { label: '中', value: 'medium' },
  { label: '低', value: 'low' },
]

/**
 * 自动播放选项
 */
const autoPlayOptions = [
  { label: '始终自动播放', value: 'always' },
  { label: '仅WiFi下', value: 'wifi' },
  { label: '从不', value: 'never' },
]
</script>

<template>
  <NCard title="媒体与存储" class="settings-card">
    <NSpace vertical size="large">
      <!-- 自动下载 -->
      <NList bordered>
        <NListItem>
          <NThing
            title="自动下载媒体"
            description="在WiFi网络下自动下载图片和视频"
          >
            <template #avatar>
              <div class="setting-icon primary">
                <Download :size="20" />
              </div>
            </template>
            <template #action>
              <NSwitch
                :value="localSettings.autoDownloadMedia"
                @update:value="(v) => updateSetting('autoDownloadMedia', v)"
              />
            </template>
          </NThing>
        </NListItem>
      </NList>

      <NDivider />

      <!-- 图片质量 -->
      <div class="settings-section">
        <h4 class="section-title">图片设置</h4>
        <NList bordered>
          <NListItem>
            <NThing title="图片质量" description="选择发送和显示的图片质量">
              <template #avatar>
                <div class="setting-icon">
                  <Image :size="18" />
                </div>
              </template>
              <template #action>
                <NSelect
                  :value="localSettings.imageQuality"
                  :options="imageQualityOptions"
                  size="small"
                  style="width: 100px"
                  @update:value="(v) => updateSetting('imageQuality', v as ImageQuality)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <NDivider />

      <!-- 自动播放 -->
      <div class="settings-section">
        <h4 class="section-title">视频设置</h4>
        <NList bordered>
          <NListItem>
            <NThing title="自动播放视频" description="选择视频自动播放的条件">
              <template #avatar>
                <div class="setting-icon">
                  <Play :size="18" />
                </div>
              </template>
              <template #action>
                <NSelect
                  :value="localSettings.autoPlayVideo"
                  :options="autoPlayOptions"
                  size="small"
                  style="width: 130px"
                  @update:value="(v) => updateSetting('autoPlayVideo', v)"
                />
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <NDivider />

      <!-- 存储管理 -->
      <div class="settings-section">
        <h4 class="section-title">存储管理</h4>
        <NList bordered>
          <NListItem>
            <NThing title="清除缓存" description="清除本地缓存的媒体文件">
              <template #avatar>
                <div class="setting-icon warning">
                  <Trash2 :size="18" />
                </div>
              </template>
              <template #action>
                <NButton size="small" @click="$emit('clearCache')">
                  清除缓存
                </NButton>
              </template>
            </NThing>
          </NListItem>
        </NList>
      </div>

      <!-- 保存按钮 -->
      <div class="card-actions">
        <NButton type="primary" :loading="saving" @click="handleSave">
          保存媒体设置
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

.setting-icon.warning {
  background: var(--color-warning-soft, rgba(255, 152, 0, 0.1));
  color: var(--color-warning, #ff9800);
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
