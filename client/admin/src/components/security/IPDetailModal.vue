<script setup lang="ts">
import { NTag, NSpace, NDescriptions, NDescriptionsItem, NDivider, NIcon } from 'naive-ui'
import { Shield, ShieldCheck, Clock, User } from 'lucide-vue-next'
import type { IPEntry, IPListType } from '@/api/security'

/**
 * 组件属性定义
 */
interface Props {
  /** IP条目信息 */
  ip: IPEntry | null
}

defineProps<Props>()

/**
 * 列表类型映射配置
 */
const listTypeConfig: Record<IPListType, { text: string; type: 'error' | 'success'; icon: typeof Shield }> = {
  blacklist: { text: '黑名单', type: 'error', icon: Shield },
  whitelist: { text: '白名单', type: 'success', icon: ShieldCheck },
}

/**
 * 格式化日期时间
 * @param dateStr ISO 8601 格式日期字符串
 * @returns 格式化后的日期时间字符串
 */
const formatDateTime = (dateStr: string | null): string => {
  if (!dateStr) return '永不过期'
  try {
    return new Date(dateStr).toLocaleString('zh-CN')
  } catch {
    return dateStr || '-'
  }
}

/**
 * 检查IP是否已过期
 * @param expiresAt 过期时间
 * @returns 是否已过期
 */
const isExpired = (expiresAt: string | null): boolean => {
  if (!expiresAt) return false
  return new Date(expiresAt) < new Date()
}

/**
 * 获取过期状态文本
 * @param expiresAt 过期时间
 * @returns 状态文本
 */
const getExpireStatus = (expiresAt: string | null): string => {
  if (!expiresAt) return '永不过期'
  if (isExpired(expiresAt)) return '已过期'
  const now = new Date()
  const expire = new Date(expiresAt)
  const diff = expire.getTime() - now.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  if (days > 0) return `${days}天后过期`
  const hours = Math.floor(diff / (1000 * 60 * 60))
  if (hours > 0) return `${hours}小时后过期`
  const minutes = Math.floor(diff / (1000 * 60))
  return `${minutes}分钟后过期`
}
</script>

<template>
  <div v-if="ip" class="ip-detail-modal">
    <!-- IP状态头部 -->
    <div class="ip-header">
      <div class="ip-address">{{ ip.ip_address }}</div>
      <NSpace size="small">
        <NTag
          :type="listTypeConfig[ip.list_type].type"
          size="large"
          :bordered="false"
        >
          <NIcon :component="listTypeConfig[ip.list_type].icon" :size="14" style="margin-right: 4px;" />
          {{ listTypeConfig[ip.list_type].text }}
        </NTag>
        <NTag v-if="isExpired(ip.expires_at)" type="error" size="large">
          已过期
        </NTag>
        <NTag v-else type="success" size="large">
          有效
        </NTag>
      </NSpace>
    </div>

    <NDivider />

    <!-- IP详细信息 -->
    <NDescriptions :columns="1" label-placement="left" label-align="right" label-style="width: 100px">
      <NDescriptionsItem label="IP地址">
        <span class="ip-text">{{ ip.ip_address }}</span>
      </NDescriptionsItem>
      <NDescriptionsItem label="列表类型">
        <NTag :type="listTypeConfig[ip.list_type].type" size="small">
          <NIcon :component="listTypeConfig[ip.list_type].icon" :size="12" style="margin-right: 4px;" />
          {{ listTypeConfig[ip.list_type].text }}
        </NTag>
      </NDescriptionsItem>
      <NDescriptionsItem label="过期时间">
        <NSpace align="center" size="small">
          <NIcon :component="Clock" :size="14" />
          <span :class="{ 'expired-text': isExpired(ip.expires_at) }">
            {{ formatDateTime(ip.expires_at) }}
          </span>
        </NSpace>
      </NDescriptionsItem>
      <NDescriptionsItem label="过期状态">
        <NTag :type="isExpired(ip.expires_at) ? 'error' : 'success'" size="small">
          {{ getExpireStatus(ip.expires_at) }}
        </NTag>
      </NDescriptionsItem>
      <NDescriptionsItem label="创建者">
        <NSpace align="center" size="small">
          <NIcon :component="User" :size="14" />
          {{ ip.created_by }}
        </NSpace>
      </NDescriptionsItem>
      <NDescriptionsItem label="创建时间">
        {{ formatDateTime(ip.created_at) }}
      </NDescriptionsItem>
    </NDescriptions>

    <NDivider />

    <!-- 备注信息 -->
    <div class="remark-section">
      <h4 class="section-title">备注</h4>
      <div class="remark-content">
        {{ ip.remark || '暂无备注' }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.ip-detail-modal {
  padding: 8px;
}

.ip-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 16px 0;
}

.ip-address {
  font-family: monospace;
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 1px;
}

.ip-text {
  font-family: monospace;
  font-size: 14px;
  font-weight: 500;
}

.section-title {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.remark-section {
  margin-top: 16px;
}

.remark-content {
  padding: 16px;
  background-color: var(--bg-secondary);
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  min-height: 60px;
}

.expired-text {
  color: var(--error-color);
  text-decoration: line-through;
}

:deep(.n-descriptions-table-content) {
  font-size: 13px;
}
</style>
