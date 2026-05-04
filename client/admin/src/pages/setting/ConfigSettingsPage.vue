<script setup lang="ts">
import { ref, computed, onMounted, h } from 'vue'
import {
  NCard,
  NInput,
  NInputNumber,
  NSwitch,
  NSelect,
  NButton,
  NTag,
  NPopconfirm,
  NEmpty,
  NSpace,
  NSpin,
  NAlert,
  useMessage,
} from 'naive-ui'
import { Save, RotateCcw, Info, Database, Settings, Shield } from 'lucide-vue-next'
import { configApi, type ConfigItem, type ConfigValueType } from '@/api/config'
import { useAuthStore } from '@/store'

/**
 * 编辑中的配置项
 */
interface EditingConfig {
  key: string
  value: string | number | boolean
  valueType: ConfigValueType
}

const message = useMessage()
const authStore = useAuthStore()

/** 配置列表 */
const configs = ref<ConfigItem[]>([])
/** 加载状态 */
const loading = ref(false)
/** 保存中状态 */
const savingKeys = ref<Set<string>>(new Set())
/** 编辑中的配置 */
const editingConfigs = ref<Map<string, EditingConfig>>(new Map())
/** 当前选中的分类 */
const selectedCategory = ref<string>('all')

/** 是否为SuperAdmin */
const isSuperAdmin = computed(() => authStore.userInfo?.role === 'super_admin')

/** 配置分类选项 */
const categoryOptions = [
  { label: '全部分类', value: 'all' },
  { label: '系统设置', value: 'system' },
  { label: '安全设置', value: 'security' },
  { label: '数据库', value: 'database' },
  { label: 'Redis', value: 'redis' },
  { label: 'WebSocket', value: 'websocket' },
  { label: '上传设置', value: 'upload' },
]

/** 分类图标映射 */
const categoryIcons: Record<string, typeof Settings> = {
  system: Settings,
  security: Shield,
  database: Database,
  redis: Database,
  websocket: Settings,
  upload: Database,
}

/** 过滤后的配置列表 */
const filteredConfigs = computed(() => {
  if (selectedCategory.value === 'all') {
    return configs.value
  }
  return configs.value.filter((config) => config.category === selectedCategory.value)
})

/** 按分类分组的配置 */
const groupedConfigs = computed(() => {
  const groups: Record<string, ConfigItem[]> = {}
  filteredConfigs.value.forEach((config) => {
    const category = config.category || 'other'
    if (!groups[category]) {
      groups[category] = []
    }
    groups[category].push(config)
  })
  return groups
})

/**
 * 获取分类显示名称
 * @param category 分类标识
 * @returns 显示名称
 */
const getCategoryLabel = (category: string): string => {
  const option = categoryOptions.find((opt) => opt.value === category)
  return option?.label || category
}

/**
 * 获取分类图标
 * @param category 分类标识
 * @returns 图标组件
 */
const getCategoryIcon = (category: string) => {
  return categoryIcons[category] || Settings
}

/**
 * 获取值类型标签
 * @param type 值类型
 * @returns 标签配置
 */
const getValueTypeConfig = (type: ConfigValueType) => {
  const configs: Record<ConfigValueType, { label: string; type: 'default' | 'primary' | 'info' | 'success' }> = {
    string: { label: '字符串', type: 'default' },
    integer: { label: '整数', type: 'primary' },
    float: { label: '浮点数', type: 'primary' },
    boolean: { label: '布尔值', type: 'success' },
    json: { label: 'JSON', type: 'info' },
  }
  return configs[type] || { label: type, type: 'default' }
}

/**
 * 获取配置项的编辑值
 * @param config 配置项
 * @returns 编辑值
 */
const getEditValue = (config: ConfigItem): string | number | boolean => {
  const editing = editingConfigs.value.get(config.key)
  if (editing) {
    return editing.value as string | number | boolean
  }
  return config.value as string | number | boolean
}

/**
 * 更新编辑值
 * @param config 配置项
 * @param value 新值
 */
const updateEditValue = (config: ConfigItem, value: string | number | boolean) => {
  editingConfigs.value.set(config.key, {
    key: config.key,
    value,
    valueType: config.value_type,
  })
}

/**
 * 检查配置项是否已修改
 * @param config 配置项
 * @returns 是否已修改
 */
const isModified = (config: ConfigItem): boolean => {
  const editing = editingConfigs.value.get(config.key)
  if (!editing) return false
  return JSON.stringify(editing.value) !== JSON.stringify(config.value)
}

/**
 * 获取配置项的输入组件
 * @param config 配置项
 */
const renderConfigInput = (config: ConfigItem) => {
  const value = getEditValue(config)
  const disabled = !config.is_editable || !isSuperAdmin.value

  switch (config.value_type) {
    case 'boolean':
      return h(NSwitch, {
        value: Boolean(value),
        onUpdateValue: (val: boolean) => updateEditValue(config, val),
        disabled,
      })

    case 'integer':
      return h(NInputNumber, {
        value: typeof value === 'number' ? value : parseInt(String(value)) || 0,
        onUpdateValue: (val: number | null) => updateEditValue(config, val ?? 0),
        disabled,
        style: { width: '200px' },
      })

    case 'float':
      return h(NInputNumber, {
        value: typeof value === 'number' ? value : parseFloat(String(value)) || 0,
        onUpdateValue: (val: number | null) => updateEditValue(config, val ?? 0),
        disabled,
        precision: 2,
        style: { width: '200px' },
      })

    case 'json':
      return h(NInput, {
        value: typeof value === 'string' ? value : JSON.stringify(value),
        onUpdateValue: (val: string) => updateEditValue(config, val),
        disabled,
        type: 'textarea',
        rows: 3,
        placeholder: '输入JSON格式数据',
      })

    case 'string':
    default:
      return h(NInput, {
        value: String(value),
        onUpdateValue: (val: string) => updateEditValue(config, val),
        disabled,
        placeholder: '输入配置值',
      })
  }
}

/**
 * 保存单个配置项
 * @param config 配置项
 */
const saveConfig = async (config: ConfigItem) => {
  const editing = editingConfigs.value.get(config.key)
  if (!editing) return

  savingKeys.value.add(config.key)
  try {
    let value: string | number | boolean | unknown = editing.value

    // 根据类型转换值
    switch (config.value_type) {
      case 'integer':
        value = parseInt(String(value)) || 0
        break
      case 'float':
        value = parseFloat(String(value)) || 0
        break
      case 'boolean':
        value = Boolean(value)
        break
      case 'json':
        try {
          value = JSON.parse(String(value))
        } catch {
          message.error('JSON格式无效')
          return
        }
        break
    }

    const response = await configApi.updateConfig(config.key, { value })
    if (response.success) {
      message.success('保存成功')
      editingConfigs.value.delete(config.key)
      // 更新本地数据
      const index = configs.value.findIndex((c) => c.key === config.key)
      if (index !== -1 && response.data) {
        configs.value[index] = response.data
      }
    } else {
      message.error(response.message || '保存失败')
    }
  } catch (error) {
    console.error('保存配置失败:', error)
    message.error('保存失败')
  } finally {
    savingKeys.value.delete(config.key)
  }
}

/**
 * 重置单个配置项到默认值
 * @param config 配置项
 */
const resetConfig = async (config: ConfigItem) => {
  savingKeys.value.add(config.key)
  try {
    const response = await configApi.updateConfig(config.key, { value: config.default_value })
    if (response.success) {
      message.success('已重置到默认值')
      editingConfigs.value.delete(config.key)
      // 更新本地数据
      const index = configs.value.findIndex((c) => c.key === config.key)
      if (index !== -1 && response.data) {
        configs.value[index] = response.data
      }
    } else {
      message.error(response.message || '重置失败')
    }
  } catch (error) {
    console.error('重置配置失败:', error)
    message.error('重置失败')
  } finally {
    savingKeys.value.delete(config.key)
  }
}

/**
 * 重置所有配置到默认值
 */
const resetAllConfigs = async () => {
  loading.value = true
  try {
    const response = await configApi.resetConfigs()
    if (response.success) {
      message.success('所有配置已重置到默认值')
      editingConfigs.value.clear()
      await fetchConfigs()
    } else {
      message.error(response.message || '重置失败')
    }
  } catch (error) {
    console.error('重置所有配置失败:', error)
    message.error('重置失败')
  } finally {
    loading.value = false
  }
}

/**
 * 获取配置列表
 */
const fetchConfigs = async () => {
  loading.value = true
  try {
    const response = await configApi.getConfigs()
    if (response.success && response.data) {
      configs.value = response.data
    } else {
      message.error('获取配置列表失败')
    }
  } catch (error) {
    console.error('获取配置列表失败:', error)
    message.error('获取配置列表失败')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchConfigs()
})
</script>

<template>
  <div class="config-settings-page">
    <NSpin :show="loading">
      <!-- 页面标题和操作栏 -->
      <div class="page-header">
        <div class="header-left">
          <h2 class="page-title">系统配置</h2>
          <p class="page-desc">管理系统配置项，仅超级管理员可修改</p>
        </div>
        <div class="header-right">
          <NPopconfirm
            v-if="isSuperAdmin"
            @positive-click="resetAllConfigs"
            :disabled="loading"
          >
            <template #trigger>
              <NButton :disabled="loading">
                <template #icon>
                  <RotateCcw :size="16" />
                </template>
                重置所有配置
              </NButton>
            </template>
            确定要将所有配置重置为默认值吗？此操作不可恢复。
          </NPopconfirm>
        </div>
      </div>

      <!-- 权限提示 -->
      <NAlert v-if="!isSuperAdmin" type="info" class="permission-alert">
        <template #icon>
          <Info :size="16" />
        </template>
        您当前以管理员身份查看，只有超级管理员可以修改配置值。
      </NAlert>

      <!-- 分类筛选 -->
      <NCard class="filter-card" :bordered="false">
        <NSelect
          v-model:value="selectedCategory"
          :options="categoryOptions"
          placeholder="选择分类"
          style="width: 200px"
        />
      </NCard>

      <!-- 配置列表 -->
      <div v-if="Object.keys(groupedConfigs).length > 0" class="config-groups">
        <NCard
          v-for="(items, category) in groupedConfigs"
          :key="category"
          class="config-group-card"
          :bordered="false"
        >
          <template #header>
            <div class="group-header">
              <component :is="getCategoryIcon(category)" :size="18" />
              <span class="group-title">{{ getCategoryLabel(category) }}</span>
              <NTag size="small" type="info">{{ items.length }}项</NTag>
            </div>
          </template>

          <div class="config-items">
            <div
              v-for="config in items"
              :key="config.key"
              class="config-item"
              :class="{ modified: isModified(config) }"
            >
              <div class="config-info">
                <div class="config-header">
                  <span class="config-key">{{ config.key }}</span>
                  <NTag size="small" :type="getValueTypeConfig(config.value_type).type">
                    {{ getValueTypeConfig(config.value_type).label }}
                  </NTag>
                  <NTag v-if="!config.is_editable" size="small" type="warning">只读</NTag>
                </div>
                <p class="config-desc">{{ config.description }}</p>
                <p class="config-default">
                  默认值: <code>{{ JSON.stringify(config.default_value) }}</code>
                </p>
              </div>

              <div class="config-value">
                <component :is="renderConfigInput(config)" />
              </div>

              <div class="config-actions">
                <NSpace>
                  <NButton
                    v-if="isModified(config)"
                    type="primary"
                    size="small"
                    :loading="savingKeys.has(config.key)"
                    :disabled="!isSuperAdmin"
                    @click="saveConfig(config)"
                  >
                    <template #icon>
                      <Save :size="14" />
                    </template>
                    保存
                  </NButton>
                  <NPopconfirm
                    v-if="isSuperAdmin"
                    @positive-click="resetConfig(config)"
                    :disabled="savingKeys.has(config.key)"
                  >
                    <template #trigger>
                      <NButton
                        size="small"
                        :disabled="savingKeys.has(config.key)"
                      >
                        <template #icon>
                          <RotateCcw :size="14" />
                        </template>
                        重置
                      </NButton>
                    </template>
                    重置为默认值？
                  </NPopconfirm>
                </NSpace>
              </div>
            </div>
          </div>
        </NCard>
      </div>

      <!-- 空状态 -->
      <NEmpty v-else description="暂无配置项" class="empty-state" />
    </NSpin>
  </div>
</template>

<style scoped>
.config-settings-page {
  padding: 16px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.header-left {
  flex: 1;
}

.page-title {
  margin: 0 0 4px;
  font-size: 20px;
  font-weight: 600;
}

.page-desc {
  margin: 0;
  color: var(--text-color-secondary);
  font-size: 14px;
}

.permission-alert {
  margin-bottom: 16px;
}

.filter-card {
  margin-bottom: 16px;
}

.config-groups {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-group-card {
  background: var(--card-color);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-title {
  font-weight: 600;
  font-size: 16px;
}

.config-items {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  border-radius: 8px;
  background: var(--body-color);
  transition: all 0.2s;
}

.config-item.modified {
  background: var(--primary-color-hover);
  border: 1px solid var(--primary-color);
}

.config-info {
  flex: 1;
  min-width: 0;
}

.config-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
  flex-wrap: wrap;
}

.config-key {
  font-weight: 600;
  font-size: 14px;
  font-family: monospace;
  color: var(--primary-color);
}

.config-desc {
  margin: 4px 0;
  color: var(--text-color-secondary);
  font-size: 13px;
}

.config-default {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--text-color-tertiary);
}

.config-default code {
  background: var(--code-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.config-value {
  width: 250px;
  flex-shrink: 0;
}

.config-actions {
  width: 120px;
  flex-shrink: 0;
  display: flex;
  justify-content: flex-end;
}

.empty-state {
  padding: 60px 0;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .config-settings-page {
    padding: 12px;
  }

  .page-header {
    flex-direction: column;
    gap: 12px;
  }

  .config-item {
    flex-direction: column;
    gap: 12px;
  }

  .config-value {
    width: 100%;
  }

  .config-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
