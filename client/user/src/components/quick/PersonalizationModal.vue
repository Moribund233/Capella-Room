<template>
  <div class="personalization-modal" :class="{ 'is-mobile': isMobile }">
    <n-tabs type="line" animated v-model:value="activeTab">
      <!-- 主题设置 -->
      <n-tab-pane name="theme" tab="主题">
        <div class="settings-group">
          <h4 class="group-title">主题模式</h4>
          <div class="theme-options">
            <div
              v-for="option in themeOptions"
              :key="option.value"
              class="theme-option"
              :class="{ active: localConfig.theme === option.value }"
              @click="localConfig.theme = option.value"
            >
              <div class="theme-preview" :class="option.value">
                <div class="preview-content"></div>
              </div>
              <span class="theme-label">{{ option.label }}</span>
            </div>
          </div>
        </div>

        <div class="settings-group">
          <h4 class="group-title">强调色</h4>
          <n-space align="center" class="accent-setting">
            <n-switch v-model:value="localConfig.enableCustomAccent" />
            <span>启用自定义强调色</span>
          </n-space>
          <div v-if="localConfig.enableCustomAccent" class="color-picker-wrapper">
            <div class="preset-colors">
              <div
                v-for="color in presetColors"
                :key="color"
                class="color-preset"
                :style="{ backgroundColor: color }"
                :class="{ active: localConfig.accentColor === color }"
                @click="localConfig.accentColor = color"
              />
            </div>
            <div class="custom-color">
              <span>自定义:</span>
              <input
                type="color"
                v-model="localConfig.accentColor"
                class="color-input"
              />
              <n-input
                v-model:value="localConfig.accentColor"
                size="small"
                style="width: 100px"
                placeholder="#07c160"
              />
            </div>
          </div>
        </div>
      </n-tab-pane>

      <!-- 背景设置 -->
      <n-tab-pane name="background" tab="背景">
        <div class="settings-group">
          <h4 class="group-title">背景不透明度</h4>
          <p class="group-desc">智能调整：根据界面元素类型自动应用不同透明度</p>
          <div class="setting-item">
            <n-slider
              v-model:value="localConfig.backgroundOpacity"
              :min="0.3"
              :max="1"
              :step="0.05"
              :format-tooltip="(v) => `${Math.round(v * 100)}%`"
            />
            <span class="setting-value">{{ Math.round(localConfig.backgroundOpacity * 100) }}%</span>
          </div>
          <div class="opacity-preview">
            <div class="preview-item">
              <span class="preview-label">容器</span>
              <div
                class="preview-bar"
                :style="{ opacity: smartOpacity.containerOpacity }"
              />
            </div>
            <div class="preview-item">
              <span class="preview-label">浮层</span>
              <div
                class="preview-bar"
                :style="{ opacity: smartOpacity.elevatedOpacity }"
              />
            </div>
            <div class="preview-item">
              <span class="preview-label">浅色背景</span>
              <div
                class="preview-bar light"
                :style="{ opacity: smartOpacity.lightBgOpacity }"
              />
            </div>
          </div>
        </div>

        <div class="settings-group">
          <h4 class="group-title">背景图片</h4>
          <n-space align="center" class="background-setting">
            <n-switch v-model:value="localConfig.enableBackgroundImage" />
            <span>启用背景图片</span>
          </n-space>

          <div v-if="localConfig.enableBackgroundImage" class="background-image-section">
            <div class="image-input">
              <n-input
                v-model:value="localConfig.backgroundImage"
                placeholder="输入图片 URL 或上传图片"
                clearable
              />
              <n-button @click="handleUploadClick">上传</n-button>
            </div>

            <div class="preset-backgrounds">
              <div
                v-for="(bg, index) in presetBackgrounds"
                :key="index"
                class="preset-bg"
                :class="{ active: localConfig.backgroundImage === bg }"
                :style="{ backgroundImage: `url(${bg})` }"
                @click="localConfig.backgroundImage = bg"
              />
            </div>

            <div v-if="localConfig.backgroundImage" class="image-preview">
              <img :src="localConfig.backgroundImage" alt="背景预览" />
              <n-button text type="error" @click="localConfig.backgroundImage = null">
                清除图片
              </n-button>
            </div>
          </div>
        </div>
      </n-tab-pane>
    </n-tabs>

    <!-- 底部操作 -->
    <div class="modal-footer">
      <n-button @click="handleReset">重置默认</n-button>
      <n-button type="primary" @click="handleConfirm">应用设置</n-button>
    </div>

    <!-- 隐藏的文件上传输入 -->
    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      style="display: none"
      @change="handleFileChange"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import {
  NTabs,
  NTabPane,
  NSpace,
  NSwitch,
  NSlider,
  NInput,
  NButton,
} from 'naive-ui'
import { useResponsive } from '@/composables/useResponsive'
import type { PersonalizationConfig } from '@/stores/personalization'
import type { ThemeType } from '@/stores/theme'

// 使用响应式布局
const { isMobile } = useResponsive()

/**
 * 组件属性定义
 */
interface Props {
  /** 当前配置 */
  modelValue: PersonalizationConfig
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新配置 */
  (e: 'update:modelValue', value: PersonalizationConfig): void
  /** 确认 */
  (e: 'confirm'): void
  /** 重置 */
  (e: 'reset'): void
}

const emit = defineEmits<Emits>()

/**
 * 当前激活的标签页
 */
const activeTab = ref('theme')

/**
 * 本地配置状态
 */
const localConfig = reactive<PersonalizationConfig>({ ...props.modelValue })

/**
 * 文件上传输入引用
 */
const fileInput = ref<HTMLInputElement | null>(null)

/**
 * 主题选项
 */
const themeOptions: { value: ThemeType; label: string }[] = [
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
  { value: 'system', label: '跟随系统' },
]

/**
 * 预设强调色
 */
const presetColors = [
  '#07c160', // 微信绿
  '#1890ff', // 钉钉蓝
  '#722ed1', // 紫色
  '#eb2f96', // 粉色
  '#f5222d', // 红色
  '#fa8c16', // 橙色
  '#52c41a', // 绿色
  '#13c2c2', // 青色
]

/**
 * 预设背景图（使用渐变色作为示例）
 */
const presetBackgrounds = [
  'https://images.unsplash.com/photo-1557683316-973673baf926?w=800&q=80',
  'https://images.unsplash.com/photo-1557682250-33bd709cbe85?w=800&q=80',
  'https://images.unsplash.com/photo-1557682260-96773eb01377?w=800&q=80',
  'https://images.unsplash.com/photo-1579546929518-9e396f3cc809?w=800&q=80',
  'https://images.unsplash.com/photo-1557682224-5b8590cd9ec5?w=800&q=80',
  'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=800&q=80',
]

/**
 * 智能透明度计算
 * 根据基础透明度计算各层级的实际透明度
 */
const smartOpacity = computed(() => {
  const base = localConfig.backgroundOpacity
  return {
    containerOpacity: Math.min(1, 0.3 + base * 0.7),
    elevatedOpacity: Math.min(1, 0.5 + base * 0.5),
    lightBgOpacity: Math.min(1, 0.2 + base * 0.4),
    darkBgOpacity: Math.min(1, 0.4 + base * 0.6),
  }
})

/**
 * 处理确认
 */
function handleConfirm(): void {
  emit('update:modelValue', { ...localConfig })
  emit('confirm')
}

/**
 * 处理重置
 */
function handleReset(): void {
  emit('reset')
}

/**
 * 触发文件上传点击
 */
function handleUploadClick(): void {
  fileInput.value?.click()
}

/**
 * 处理文件选择
 */
function handleFileChange(event: Event): void {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    const reader = new FileReader()
    reader.onload = (e) => {
      localConfig.backgroundImage = e.target?.result as string
    }
    reader.readAsDataURL(file)
  }
}
</script>

<style scoped>
.personalization-modal {
  padding: 16px 0;
  width: 480px;
  max-width: 90vw;
}

/* 移动端适配 */
.personalization-modal.is-mobile {
  width: 100vw;
  max-width: 100vw;
  padding: 12px 0;
}

.settings-group {
  margin-bottom: 24px;
}

.group-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 4px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border-light);
}

.group-desc {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 12px;
}

/* 透明度预览 */
.opacity-preview {
  margin-top: 16px;
  padding: 12px;
  background-color: var(--color-background-light);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-label {
  font-size: 12px;
  color: var(--color-text-secondary);
  min-width: 60px;
}

.preview-bar {
  flex: 1;
  height: 20px;
  background-color: var(--color-background);
  border-radius: 4px;
  border: 1px solid var(--color-border);
}

.preview-bar.light {
  background-color: var(--color-primary-light);
}

/* 主题选项 */
.theme-options {
  display: flex;
  gap: 16px;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s;
}

.theme-option:hover {
  background-color: var(--color-background-light);
}

.theme-option.active {
  background-color: var(--color-primary-light);
}

.theme-preview {
  width: 64px;
  height: 48px;
  border-radius: 6px;
  border: 2px solid var(--color-border);
  overflow: hidden;
  position: relative;
}

.theme-option.active .theme-preview {
  border-color: var(--color-primary);
}

.theme-preview.light {
  background: linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%);
}

.theme-preview.dark {
  background: linear-gradient(135deg, #1a1a1a 0%, #0a0a0a 100%);
}

.theme-preview.system {
  background: linear-gradient(135deg, #fafafa 50%, #1a1a1a 50%);
}

.preview-content {
  position: absolute;
  bottom: 8px;
  left: 8px;
  right: 8px;
  height: 8px;
  background: var(--color-primary);
  border-radius: 2px;
}

.theme-label {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.theme-option.active .theme-label {
  color: var(--color-primary);
}

/* 强调色设置 */
.accent-setting {
  margin-bottom: 16px;
}

.color-picker-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.preset-colors {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.color-preset {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.color-preset:hover {
  transform: scale(1.1);
}

.color-preset.active {
  border-color: var(--color-text-primary);
  box-shadow: 0 0 0 2px var(--color-background), 0 0 0 4px var(--color-text-primary);
}

.custom-color {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border-light);
}

.color-input {
  width: 40px;
  height: 32px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  background: none;
}

/* 背景设置 */
.background-setting {
  margin-bottom: 16px;
}

.background-image-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.image-input {
  display: flex;
  gap: 8px;
}

.preset-backgrounds {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.preset-bg {
  aspect-ratio: 16 / 9;
  border-radius: 6px;
  background-size: cover;
  background-position: center;
  cursor: pointer;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.preset-bg:hover {
  transform: scale(1.02);
}

.preset-bg.active {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-background), 0 0 0 4px var(--color-primary);
}

.image-preview {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background-color: var(--color-background-light);
  border-radius: 8px;
}

.image-preview img {
  max-width: 100%;
  max-height: 150px;
  border-radius: 4px;
  object-fit: cover;
}

/* 通用设置项 */
.setting-item {
  display: flex;
  align-items: center;
  gap: 16px;
}

.setting-item .n-slider {
  flex: 1;
}

.setting-value {
  min-width: 48px;
  text-align: right;
  font-size: 14px;
  color: var(--color-text-secondary);
}

/* 底部操作 */
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 16px;
  margin-top: 16px;
  border-top: 1px solid var(--color-border-light);
}

/* 移动端样式优化 */
.personalization-modal.is-mobile .settings-group {
  margin-bottom: 16px;
  padding: 0 12px;
}

.personalization-modal.is-mobile .group-title {
  font-size: 13px;
}

.personalization-modal.is-mobile .theme-options {
  gap: 8px;
  justify-content: space-around;
}

.personalization-modal.is-mobile .theme-option {
  padding: 4px;
}

.personalization-modal.is-mobile .theme-preview {
  width: 48px;
  height: 36px;
}

.personalization-modal.is-mobile .theme-label {
  font-size: 11px;
}

.personalization-modal.is-mobile .preset-colors {
  gap: 6px;
}

.personalization-modal.is-mobile .color-preset {
  width: 28px;
  height: 28px;
}

.personalization-modal.is-mobile .custom-color {
  gap: 8px;
  font-size: 12px;
}

.personalization-modal.is-mobile .color-input {
  width: 32px;
  height: 28px;
}

.personalization-modal.is-mobile .preset-backgrounds {
  grid-template-columns: repeat(2, 1fr);
  gap: 6px;
}

.personalization-modal.is-mobile .opacity-preview {
  padding: 8px;
}

.personalization-modal.is-mobile .preview-label {
  min-width: 50px;
  font-size: 11px;
}

.personalization-modal.is-mobile .modal-footer {
  padding: 12px;
  margin-top: 12px;
  gap: 8px;
}
</style>
