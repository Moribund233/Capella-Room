<template>
  <div class="ui-settings-panel">
    <n-card title="界面设置" class="settings-card">
      <n-tabs type="line" animated>
        <!-- 应用信息 -->
        <n-tab-pane name="app" tab="应用信息">
          <n-form :model="config.app" label-placement="left" label-width="120px">
            <n-form-item label="应用名称">
              <n-input v-model:value="config.app.name" placeholder="输入应用名称" />
            </n-form-item>
            <n-form-item label="Logo">
              <n-input v-model:value="config.app.logo" placeholder="输入Logo路径" />
            </n-form-item>
            <n-form-item label="版本">
              <n-input v-model:value="config.app.version" placeholder="输入版本号" disabled />
            </n-form-item>
          </n-form>
        </n-tab-pane>

        <!-- 主题设置 -->
        <n-tab-pane name="theme" tab="主题">
          <n-form :model="config.theme" label-placement="left" label-width="120px">
            <n-form-item label="默认主题">
              <n-radio-group v-model:value="config.theme.name">
                <n-radio-button value="light">浅色</n-radio-button>
                <n-radio-button value="dark">深色</n-radio-button>
              </n-radio-group>
            </n-form-item>
          </n-form>
        </n-tab-pane>

        <!-- 侧边栏设置 -->
        <n-tab-pane name="sidebar" tab="侧边栏">
          <div class="items-list">
            <div v-for="(item, index) in config.sidebar.items" :key="index" class="item-row">
              <div class="item-info">
                <n-input v-model:value="item.name" placeholder="名称" class="item-input" />
                <n-input v-model:value="item.path" placeholder="路径" class="item-input" disabled />
              </div>
              <div class="item-icon">
                <n-button quaternary @click="openIconPicker('sidebar', index)">
                  <component :is="getIconComponent(item.icon)" :size="18" />
                </n-button>
                <span class="icon-label">{{ item.icon }}</span>
              </div>
              <n-button-group>
                <n-button quaternary :disabled="index === 0" @click="moveItem('sidebar', index, -1)">
                  <ArrowUp :size="16" />
                </n-button>
                <n-button quaternary :disabled="index === config.sidebar.items.length - 1"
                  @click="moveItem('sidebar', index, 1)">
                  <ArrowDown :size="16" />
                </n-button>
              </n-button-group>
            </div>
          </div>
        </n-tab-pane>

        <!-- 快捷栏设置 -->
        <n-tab-pane name="quickbar" tab="快捷栏">
          <div class="items-list">
            <div v-for="(item, index) in quickBarItems" :key="index" class="item-row">
              <div class="item-info">
                <n-input v-model:value="item.label" placeholder="标签" class="item-input" />
                <n-tag :type="item.display === 'visible' ? 'success' : 'default'">
                  {{ item.display === 'visible' ? '外显' : '下拉' }}
                </n-tag>
              </div>
              <div class="item-icon">
                <n-button quaternary @click="openIconPicker('quickbar', index)">
                  <component :is="getIconComponent(item.icon)" :size="18" />
                </n-button>
                <span class="icon-label">{{ item.icon }}</span>
              </div>
              <n-button-group>
                <n-button quaternary :disabled="index === 0" @click="moveItem('quickbar', index, -1)">
                  <ArrowUp :size="16" />
                </n-button>
                <n-button quaternary :disabled="index === quickBarItems.length - 1"
                  @click="moveItem('quickbar', index, 1)">
                  <ArrowDown :size="16" />
                </n-button>
              </n-button-group>
            </div>
          </div>
        </n-tab-pane>
      </n-tabs>

      <!-- 操作按钮 -->
      <div class="actions">
        <n-button type="primary" @click="saveConfig" :loading="saving">
          <template #icon>
            <Save :size="16" />
          </template>
          保存配置
        </n-button>
        <n-button @click="resetConfig">
          <template #icon>
            <RotateCcw :size="16" />
          </template>
          重置
        </n-button>
      </div>
    </n-card>

    <!-- 图标选择器弹窗 -->
    <n-modal v-model:show="showIconPicker" title="选择图标" preset="card" class="icon-picker-modal"
      :style="{ width: '340px', maxHeight: '400px' }">
      <IconPicker :selected-icon="currentEditingIcon" @select="handleIconSelect" />
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import {
  NCard,
  NTabs,
  NTabPane,
  NForm,
  NFormItem,
  NInput,
  NRadioGroup,
  NRadioButton,
  NButton,
  NButtonGroup,
  NTag,
  NModal,
  useMessage,
  useDialog,
} from 'naive-ui'
import { Save, RotateCcw, ArrowUp, ArrowDown } from 'lucide-vue-next'
import * as LucideIcons from 'lucide-vue-next'
import { useUIStore } from '@/store'
import { uiConfig as defaultUiConfig } from '@/config/ui'
import IconPicker from '@/components/common/IconPicker.vue'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import type { UIConfig, QuickItemConfig } from '@/config'

const message = useMessage()
const dialog = useDialog()
const uiStore = useUIStore()

/** 默认配置（用于重置） */
const defaultConfig = defaultUiConfig

/** 当前编辑的配置（副本） */
const config = reactive<UIConfig>({
  app: { ...uiStore.appConfig },
  sidebar: {
    items: uiStore.sidebarConfig.items.map(item => ({ ...item })),
  },
  theme: { ...uiStore.themeConfig },
  dock: {},
  quickBar: uiStore.quickBarConfig.map(item => ({
    ...item,
    children: item.children?.map(c => ({ ...c })) ?? undefined,
  })),
})

/** 类型安全的 quickBar 数组 */
const quickBarItems = computed<QuickItemConfig[]>(() => config.quickBar ?? [])

/** 保存中状态 */
const saving = ref(false)

/** 图标选择器显示状态 */
const showIconPicker = ref(false)

/** 当前编辑的图标 */
const currentEditingType = ref<'sidebar' | 'quickbar'>('sidebar')
const currentEditingIndex = ref(0)
const currentEditingIcon = ref('')

/**
 * 获取图标组件
 * @param iconName 图标名称
 * @returns 图标组件
 */
function getIconComponent(iconName: string): FunctionalComponent<LucideProps> {
  return (LucideIcons as unknown as Record<string, FunctionalComponent<LucideProps>>)[iconName]
    || LucideIcons.Circle
}

/**
 * 打开图标选择器
 * @param type 编辑类型
 * @param index 索引
 */
function openIconPicker(type: 'sidebar' | 'quickbar', index: number): void {
  currentEditingType.value = type
  currentEditingIndex.value = index
  if (type === 'sidebar') {
    const item = config.sidebar.items[index]
    if (item) {
      currentEditingIcon.value = item.icon
    }
  } else {
    const item = quickBarItems.value[index]
    if (item) {
      currentEditingIcon.value = item.icon
    }
  }
  showIconPicker.value = true
}

/**
 * 处理图标选择
 * @param iconName 图标名称
 */
function handleIconSelect(iconName: string): void {
  if (currentEditingType.value === 'sidebar') {
    const item = config.sidebar.items[currentEditingIndex.value]
    if (item) {
      item.icon = iconName
    }
  } else {
    const item = quickBarItems.value[currentEditingIndex.value]
    if (item) {
      item.icon = iconName
    }
  }
  showIconPicker.value = false
  message.success('图标已更新')
}

/**
 * 移动项目
 * @param type 类型
 * @param index 当前索引
 * @param direction 移动方向 (-1: 上移, 1: 下移)
 */
function moveItem(type: 'sidebar' | 'quickbar', index: number, direction: number): void {
  const items = type === 'sidebar' ? config.sidebar.items : config.quickBar
  if (!items) return

  const newIndex = index + direction
  if (newIndex < 0 || newIndex >= items.length) return

  const currentItem = items[index]
  const targetItem = items[newIndex]
  if (!currentItem || !targetItem) return

  items[index] = targetItem
  items[newIndex] = currentItem
}

/**
 * 保存配置到 Store
 */
async function saveConfig(): Promise<void> {
  saving.value = true
  try {
    // 更新到 UI Store
    uiStore.updateAppConfig(config.app)
    uiStore.updateThemeConfig(config.theme)
    uiStore.updateSidebarItems(config.sidebar.items)
    uiStore.updateQuickBarItems(config.quickBar ?? [])

    message.success('配置已保存，刷新页面后生效')
  } catch (error) {
    message.error('保存失败')
    console.error(error)
  } finally {
    saving.value = false
  }
}

/**
 * 重置配置
 */
function resetConfig(): void {
  dialog.warning({
    title: '确认重置',
    content: '确定要将所有配置重置为默认值吗？此操作不可恢复。',
    positiveText: '确认重置',
    negativeText: '取消',
    onPositiveClick: () => {
      // 重置 Store
      uiStore.resetToDefault()

      // 重置本地编辑状态为默认值
      config.app = { ...defaultConfig.app }
      config.theme = { ...defaultConfig.theme }
      config.sidebar.items = defaultConfig.sidebar.items.map(item => ({ ...item }))
      config.quickBar = defaultConfig.quickBar?.map(item => ({
        ...item,
        children: item.children?.map(c => ({ ...c })) ?? undefined,
      })) ?? []

      message.success('配置已重置为默认值')
    },
  })
}
</script>

<style scoped>
.ui-settings-panel {
  padding: 24px;
}

.settings-card {
  max-width: 900px;
  margin: 0 auto;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-layout);
  border-radius: 8px;
}

.item-info {
  flex: 1;
  display: flex;
  gap: 8px;
}

.item-input {
  flex: 1;
}

.item-icon {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  min-width: 80px;
}

.icon-label {
  font-size: 10px;
  color: var(--text-secondary);
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
}

.icon-picker-modal {
  width: 340px !important;
  max-width: 90vw;
}
</style>
