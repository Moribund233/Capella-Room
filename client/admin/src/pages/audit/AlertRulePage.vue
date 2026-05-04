<script setup lang="ts">
import { ref, h, onMounted } from 'vue'
import {
  NCard,
  NTable,
  NTag,
  NSpace,
  NButton,
  NSwitch,
  NEmpty,
  NModal,
  useMessage,
} from 'naive-ui'
import { Ruler, Edit2, RefreshCw } from 'lucide-vue-next'
import { RuleEditor } from '@/components/audit'
import { MobileTableCard } from '@/components/common'
import { useStatusBar } from '@/composables'
import { useLayoutStore } from '@/store/layout'
import { auditApi, type AlertRule, type UpdateAlertRuleRequest } from '@/api/audit'
import type { MobileColumn, MobileAction } from '@/components/common'

const message = useMessage()
const { setContent } = useStatusBar()
const layoutStore = useLayoutStore()
const { isMobile } = layoutStore

// ==================== 数据状态 ====================

/** 规则列表数据 */
const data = ref<AlertRule[]>([])
/** 加载状态 */
const loading = ref(false)
/** 编辑对话框显示状态 */
const showEditModal = ref(false)
/** 当前编辑的规则 */
const editingRule = ref<AlertRule | null>(null)
/** 保存中状态 */
const saving = ref(false)

// ==================== 数据获取 ====================

/**
 * 获取告警规则列表
 */
const fetchAlertRules = async () => {
  loading.value = true

  try {
    const response = await auditApi.getAlertRules()

    if (response.success && response.data) {
      data.value = response.data.rules
      return true
    }
    return false
  } catch (error) {
    console.error('获取告警规则失败:', error)
    message.error('获取告警规则失败')
    return false
  } finally {
    loading.value = false
  }
}

/**
 * 刷新列表
 */
const refresh = async () => {
  const success = await fetchAlertRules()
  if (success) {
    updateStatusBar()
  }
  return success
}

// ==================== 事件处理 ====================

/**
 * 更新状态栏
 */
const updateStatusBar = () => {
  setContent([
    h(Ruler, { size: 14, style: { marginRight: '6px' } }),
    ` 共 ${data.value.length} 条规则`,
  ])
}

/**
 * 处理刷新
 */
const handleRefresh = async () => {
  const success = await refresh()
  if (success) {
    message.success('刷新成功')
  }
}

/**
 * 打开编辑对话框
 */
const handleEdit = (rule: AlertRule) => {
  editingRule.value = rule
  showEditModal.value = true
}

/**
 * 处理保存规则
 */
const handleSave = async (id: string, updateData: UpdateAlertRuleRequest) => {
  saving.value = true

  try {
    const response = await auditApi.updateAlertRule(id, updateData)

    if (response.success) {
      message.success('规则更新成功')
      showEditModal.value = false
      editingRule.value = null
      await refresh()
    } else {
      message.error(response.message || '更新失败')
    }
  } catch {
    message.error('更新规则失败')
  } finally {
    saving.value = false
  }
}

/**
 * 处理取消编辑
 */
const handleCancelEdit = () => {
  showEditModal.value = false
  editingRule.value = null
}

/**
 * 切换规则启用状态
 */
const handleToggleEnabled = async (rule: AlertRule) => {
  try {
    const response = await auditApi.updateAlertRule(rule.id, {
      enabled: !rule.enabled,
    })

    if (response.success) {
      message.success(rule.enabled ? '已禁用规则' : '已启用规则')
      await refresh()
    } else {
      message.error(response.message || '操作失败')
    }
  } catch {
    message.error('切换规则状态失败')
  }
}

// ==================== 表格列配置 ====================

/**
 * 严重级别映射配置
 */
const severityConfig: Record<string, { text: string; type: 'default' | 'info' | 'warning' | 'error' | 'success' }> = {
  info: { text: '信息', type: 'info' },
  warning: { text: '警告', type: 'warning' },
  error: { text: '错误', type: 'error' },
  critical: { text: '严重', type: 'error' },
}

/**
 * 格式化日期时间
 */
const formatDateTime = (dateString: string): string => {
  const date = new Date(dateString)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

// ==================== 移动端表格配置 ====================

/**
 * 移动端表格列配置
 */
const mobileColumns: MobileColumn<AlertRule>[] = [
  {
    key: 'event_type',
    title: '事件类型',
    render: (rule: AlertRule) => rule.event_type || '全部事件',
  },
  {
    key: 'severity',
    title: '严重级别',
    render: (rule: AlertRule) => severityConfig[rule.severity]?.text || rule.severity,
  },
  {
    key: 'cooldown_minutes',
    title: '冷却时间',
    render: (rule: AlertRule) => `${rule.cooldown_minutes} 分钟`,
  },
  {
    key: 'notify_admins',
    title: '通知管理员',
    render: (rule: AlertRule) => (rule.notify_admins ? '是' : '否'),
  },
  {
    key: 'enabled',
    title: '状态',
    render: (rule: AlertRule) => (rule.enabled ? '启用' : '禁用'),
  },
  {
    key: 'updated_at',
    title: '更新时间',
    render: (rule: AlertRule) => formatDateTime(rule.updated_at),
  },
]

/**
 * 移动端表格操作配置
 */
const mobileActions: MobileAction<AlertRule>[] = [
  {
    label: '编辑',
    type: 'primary',
    onClick: (rule: AlertRule) => handleEdit(rule),
  },
  {
    label: '启用',
    type: 'success',
    show: (rule: AlertRule) => !rule.enabled,
    onClick: (rule: AlertRule) => handleToggleEnabled(rule),
  },
  {
    label: '禁用',
    type: 'warning',
    show: (rule: AlertRule) => rule.enabled,
    onClick: (rule: AlertRule) => handleToggleEnabled(rule),
  },
]

// ==================== 生命周期 ====================

onMounted(async () => {
  const success = await fetchAlertRules()
  if (success) updateStatusBar()
})
</script>

<template>
  <div class="alert-rule-page">
    <div class="page-header">
      <h1 class="page-title">告警规则</h1>
      <p class="page-description">管理安全告警规则，配置触发条件和通知策略</p>
    </div>

    <!-- 工具栏 -->
    <NCard class="toolbar-card" :bordered="false">
      <NSpace justify="end">
        <NButton @click="handleRefresh" :loading="loading">
          <template #icon>
            <RefreshCw :size="16" />
          </template>
          刷新
        </NButton>
      </NSpace>
    </NCard>

    <!-- 规则列表 - 桌面端 -->
    <NCard v-if="!isMobile" class="table-card" :bordered="false">
      <NTable :bordered="false" :single-line="false" size="small">
        <thead>
          <tr>
            <th>规则名称</th>
            <th>事件类型</th>
            <th>严重级别</th>
            <th>冷却时间</th>
            <th>通知管理员</th>
            <th>状态</th>
            <th>更新时间</th>
            <th style="width: 120px; text-align: center;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in data" :key="rule.id">
            <td>
              <div class="rule-name">{{ rule.name }}</div>
              <div v-if="rule.description" class="rule-desc">{{ rule.description }}</div>
            </td>
            <td>
              <NTag v-if="rule.event_type" type="default" size="small">
                {{ rule.event_type }}
              </NTag>
              <NTag v-else type="default" size="small">全部事件</NTag>
            </td>
            <td>
              <NTag :type="severityConfig[rule.severity]?.type || 'default'" size="small">
                {{ severityConfig[rule.severity]?.text || rule.severity }}
              </NTag>
            </td>
            <td>{{ rule.cooldown_minutes }} 分钟</td>
            <td>
              <NTag :type="rule.notify_admins ? 'success' : 'default'" size="small">
                {{ rule.notify_admins ? '是' : '否' }}
              </NTag>
            </td>
            <td>
              <NSwitch
                :value="rule.enabled"
                @update:value="() => handleToggleEnabled(rule)"
              >
                <template #checked>启用</template>
                <template #unchecked>禁用</template>
              </NSwitch>
            </td>
            <td>{{ formatDateTime(rule.updated_at) }}</td>
            <td style="text-align: center;">
              <NButton
                size="small"
                type="primary"
                tertiary
                @click="handleEdit(rule)"
              >
                <template #icon>
                  <Edit2 :size="14" />
                </template>
                编辑
              </NButton>
            </td>
          </tr>
        </tbody>
      </NTable>

      <div v-if="data.length === 0 && !loading" class="empty-state">
        <NEmpty description="暂无告警规则" />
      </div>
    </NCard>

    <!-- 规则列表 - 移动端 -->
    <NCard v-else class="table-card" :bordered="false">
      <MobileTableCard
        :data="data"
        :columns="mobileColumns"
        :actions="mobileActions"
        title-column="name"
        empty-text="暂无告警规则"
      />
    </NCard>

    <!-- 编辑对话框 -->
    <NModal
      v-model:show="showEditModal"
      title="编辑告警规则"
      preset="card"
      style="width: 600px; max-width: calc(100vw - 32px)"
      :mask-closable="false"
    >
      <RuleEditor
        :rule="editingRule"
        :loading="saving"
        @save="handleSave"
        @cancel="handleCancelEdit"
      />
    </NModal>
  </div>
</template>

<style scoped>
.alert-rule-page {
  padding: 24px;
  min-height: 100%;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 28px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: var(--text-primary);
}

.page-description {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
}

.toolbar-card {
  margin-bottom: 16px;
}

.table-card {
  margin-bottom: 24px;
}

.rule-name {
  font-weight: 500;
  font-size: 14px;
}

.rule-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.empty-state {
  padding: 48px 0;
  display: flex;
  justify-content: center;
}

@media (max-width: 768px) {
  .alert-rule-page {
    padding: 16px;
  }

  .page-title {
    font-size: 24px;
  }
}
</style>
