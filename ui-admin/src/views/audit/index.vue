<script setup lang="ts">
/**
 * 审计系统主页面
 * 包含统计概览和三个标签页：审计日志、安全告警、告警规则
 */

import { ref, onMounted } from 'vue'
import { useAuditLogs } from '@/composables/useAudit'
import AuditLogTab from './AuditLogTab.vue'
import AlertTab from './AlertTab.vue'
import RuleTab from './RuleTab.vue'
import { Document, Calendar, Clock, Warning, SetUp } from '@element-plus/icons-vue'

const activeTab = ref<'logs' | 'alerts' | 'rules'>('logs')

const { stats, fetchStats } = useAuditLogs()

onMounted(() => {
  fetchStats()
})

function formatNumber(num: number | undefined): string {
  if (num === undefined || num === null) return '-'
  return num.toLocaleString('zh-CN')
}
</script>

<template>
  <div class="audit-page">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">审计系统</h1>
      <p class="page-subtitle">系统操作审计、安全监控与告警管理</p>
    </div>

    <!-- 统计概览 -->
    <div class="audit-overview">
      <div class="audit-stat-card">
        <div class="audit-stat-icon audit-stat-icon--info">
          <Document class="audit-stat-svg-icon" />
        </div>
        <div class="audit-stat-info">
          <span class="audit-stat-value">{{ formatNumber(stats?.total_logs) }}</span>
          <span class="audit-stat-label">总日志数</span>
        </div>
      </div>
      <div class="audit-stat-card">
        <div class="audit-stat-icon audit-stat-icon--success">
          <Calendar class="audit-stat-svg-icon" />
        </div>
        <div class="audit-stat-info">
          <span class="audit-stat-value">{{ formatNumber(stats?.today_logs) }}</span>
          <span class="audit-stat-label">今日日志</span>
        </div>
      </div>
      <div class="audit-stat-card">
        <div class="audit-stat-icon audit-stat-icon--warning">
          <Clock class="audit-stat-svg-icon" />
        </div>
        <div class="audit-stat-info">
          <span class="audit-stat-value">{{ formatNumber(stats?.week_logs) }}</span>
          <span class="audit-stat-label">本周日志</span>
        </div>
      </div>
      <div class="audit-stat-card">
        <div class="audit-stat-icon audit-stat-icon--error">
          <Warning class="audit-stat-svg-icon" />
        </div>
        <div class="audit-stat-info">
          <span class="audit-stat-value">{{ formatNumber(stats?.month_logs) }}</span>
          <span class="audit-stat-label">本月日志</span>
        </div>
      </div>
    </div>

    <!-- 标签页导航 -->
    <div class="audit-tabs">
      <button
        class="audit-tab"
        :class="{ 'audit-tab--active': activeTab === 'logs' }"
        @click="activeTab = 'logs'"
      >
        <Document class="audit-tab-icon" />
        审计日志
      </button>
      <button
        class="audit-tab"
        :class="{ 'audit-tab--active': activeTab === 'alerts' }"
        @click="activeTab = 'alerts'"
      >
        <Warning class="audit-tab-icon" />
        安全告警
      </button>
      <button
        class="audit-tab"
        :class="{ 'audit-tab--active': activeTab === 'rules' }"
        @click="activeTab = 'rules'"
      >
        <SetUp class="audit-tab-icon" />
        告警规则
      </button>
    </div>

    <!-- 标签页内容 -->
    <AuditLogTab v-if="activeTab === 'logs'" />
    <AlertTab v-else-if="activeTab === 'alerts'" />
    <RuleTab v-else-if="activeTab === 'rules'" />
  </div>
</template>

<style>
/* 导入审计系统全局样式 */
@import '@/style/audit.css';
</style>

<style scoped>
/* 页面容器 */
.audit-page {
  padding: var(--spacing-6);
}

/* 页面标题 */
.page-header {
  margin-bottom: var(--spacing-6);
}

.page-title {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-2);
}

.page-subtitle {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
}

/* 标签页导航 */
.audit-tabs {
  display: flex;
  gap: var(--spacing-1);
  border-bottom: 1px solid var(--border-primary);
  margin-top: var(--spacing-6);
}

.audit-tab {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3) var(--spacing-5);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-secondary);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.audit-tab:hover {
  color: var(--text-primary);
  background-color: var(--bg-secondary);
}

.audit-tab--active {
  color: var(--primary);
  border-bottom-color: var(--primary);
}

.audit-tab--active:hover {
  background-color: transparent;
}

/* 图标样式 */
.audit-stat-svg-icon {
  width: 24px;
  height: 24px;
  fill: currentColor;
}

.audit-tab-icon {
  width: 16px;
  height: 16px;
  fill: currentColor;
}
</style>
