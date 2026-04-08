<script setup lang="ts">
/**
 * 告警规则标签页
 * 提供告警规则管理和配置功能
 */

import { ref, onMounted } from 'vue'
import { useAlertRules } from '@/composables/useAudit'
import { type AlertRule, type UpdateAlertRuleRequest } from '@/types'
import { List } from '@element-plus/icons-vue'

const { rules, loading, error, fetchRules, updateRule, toggleRule } = useAlertRules()

const editVisible = ref(false)
const editingRule = ref<AlertRule | null>(null)
const editForm = ref<UpdateAlertRuleRequest>({})

onMounted(() => {
  fetchRules()
})

function openEdit(rule: AlertRule) {
  editingRule.value = rule
  editForm.value = {
    enabled: rule.enabled,
    severity: rule.severity,
    cooldown_minutes: rule.cooldown_minutes,
    max_alerts_per_hour: rule.max_alerts_per_hour,
  }
  editVisible.value = true
}

async function handleSave() {
  if (!editingRule.value) return

  const success = await updateRule(editingRule.value.id, editForm.value)
  if (success) {
    editVisible.value = false
    editingRule.value = null
  }
}

async function handleToggle(rule: AlertRule) {
  await toggleRule(rule.id, !rule.enabled)
}

function getSeverityClass(severity: string): string {
  switch (severity) {
    case 'critical':
    case 'high':
      return 'severity-high'
    case 'medium':
      return 'severity-medium'
    case 'low':
      return 'severity-low'
    default:
      return 'severity-info'
  }
}

function formatDateTime(datetime: string): string {
  return new Date(datetime).toLocaleString('zh-CN')
}
</script>

<template>
  <div class="audit-tab-content">
    <!-- 错误提示 -->
    <div v-if="error" class="audit-error">
      <span class="audit-error-text">{{ error }}</span>
      <div class="audit-error-actions">
        <button class="btn btn-primary" @click="fetchRules">重试</button>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-else-if="loading && rules.length === 0" class="audit-loading">
      <div class="audit-loading-spinner"></div>
      <span>加载中...</span>
    </div>

    <!-- 空状态 -->
    <div v-else-if="rules.length === 0" class="audit-empty">
      <el-icon class="audit-empty-icon" :size="48"><List /></el-icon>
      <span class="audit-empty-text">暂无告警规则</span>
      <span class="audit-empty-hint">请联系管理员配置告警规则</span>
    </div>

    <!-- 数据表格 -->
    <div v-else class="audit-table-wrapper">
      <table class="audit-table">
        <thead>
          <tr>
            <th>规则名称</th>
            <th>描述</th>
            <th>事件类型</th>
            <th class="audit-table-col--severity">严重级别</th>
            <th>冷却时间(分钟)</th>
            <th>每小时上限</th>
            <th>状态</th>
            <th class="audit-table-col--actions">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="rule in rules" :key="rule.id">
            <td>{{ rule.name }}</td>
            <td>{{ rule.description || '-' }}</td>
            <td>{{ rule.event_type }}</td>
            <td class="audit-table-col--severity">
              <span :class="getSeverityClass(rule.severity)">{{ rule.severity }}</span>
            </td>
            <td>{{ rule.cooldown_minutes }}</td>
            <td>{{ rule.max_alerts_per_hour }}</td>
            <td>
              <label class="audit-switch">
                <input
                  type="checkbox"
                  class="audit-switch-input"
                  :checked="rule.enabled"
                  @change="handleToggle(rule)"
                />
                <span class="audit-switch-slider"></span>
                <span class="audit-switch-label">{{ rule.enabled ? '启用' : '禁用' }}</span>
              </label>
            </td>
            <td class="audit-table-col--actions">
              <button class="btn btn-sm btn-ghost" @click="openEdit(rule)">编辑</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 编辑弹窗 -->
    <div v-if="editVisible" class="modal" @click.self="editVisible = false">
      <div class="modal-content">
        <div class="modal-header">
          <h3>编辑告警规则</h3>
          <button class="btn btn-ghost" @click="editVisible = false">✕</button>
        </div>
        <div class="modal-body" v-if="editingRule">
          <div class="form-group">
            <label class="form-label">规则名称</label>
            <input type="text" class="form-input" :value="editingRule.name" disabled />
          </div>
          <div class="form-group">
            <label class="form-label">描述</label>
            <input type="text" class="form-input" :value="editingRule.description || '-'" disabled />
          </div>
          <div class="form-group">
            <label class="form-label">严重级别</label>
            <select v-model="editForm.severity" class="form-select">
              <option value="critical">严重</option>
              <option value="high">高</option>
              <option value="medium">中</option>
              <option value="low">低</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">冷却时间（分钟）</label>
            <input
              type="number"
              class="form-input"
              v-model.number="editForm.cooldown_minutes"
              min="0"
              max="1440"
            />
            <span class="form-hint">同一规则触发告警的最小间隔时间</span>
          </div>
          <div class="form-group">
            <label class="form-label">每小时告警上限</label>
            <input
              type="number"
              class="form-input"
              v-model.number="editForm.max_alerts_per_hour"
              min="1"
              max="1000"
            />
            <span class="form-hint">每小时最多触发该规则的告警数量</span>
          </div>
          <div class="form-group">
            <label class="form-label">启用状态</label>
            <label class="audit-switch">
              <input
                type="checkbox"
                class="audit-switch-input"
                v-model="editForm.enabled"
              />
              <span class="audit-switch-slider"></span>
              <span class="audit-switch-label">{{ editForm.enabled ? '启用' : '禁用' }}</span>
            </label>
          </div>
          <div class="modal-actions">
            <button class="btn btn-primary" @click="handleSave">保存</button>
            <button class="btn btn-outline" @click="editVisible = false">取消</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/* 导入审计系统全局样式 */
@import '@/style/audit.css';
</style>

<style scoped>
/* 表单样式 */
.form-group {
  margin-bottom: var(--spacing-4);
}

.form-label {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-primary);
  margin-bottom: var(--spacing-2);
}

.form-input,
.form-select {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-3);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: var(--font-size-sm);
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--primary);
}

.form-input:disabled {
  background-color: var(--bg-secondary);
  color: var(--text-secondary);
  cursor: not-allowed;
}

.form-hint {
  display: block;
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  margin-top: var(--spacing-1);
}

/* 弹窗 */
.modal {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
  padding: var(--spacing-4);
}

.modal-content {
  background-color: var(--bg-card);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  max-width: 500px;
  width: 100%;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4) var(--spacing-6);
  border-bottom: 1px solid var(--border-secondary);
}

.modal-header h3 {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
}

.modal-body {
  padding: var(--spacing-6);
}

.modal-actions {
  display: flex;
  gap: var(--spacing-2);
  margin-top: var(--spacing-6);
  padding-top: var(--spacing-4);
  border-top: 1px solid var(--border-secondary);
}
</style>
