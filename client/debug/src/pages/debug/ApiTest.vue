<template>
  <div class="api-test-page">
    <div class="page-header">
      <div class="header-title">
        <Terminal :size="24" />
        <h1>API 测试工具</h1>
      </div>
      <p class="header-desc">快速测试 HTTP API 端点，支持认证、参数配置和响应查看</p>
    </div>

    <div class="page-content">
      <!-- 左侧：端点选择器 -->
      <div class="sidebar">
        <ApiEndpointSelect :endpoints="endpoints" :grouped-endpoints="groupedEndpoints"
          :selected-endpoint="selectedEndpoint" @select="selectEndpoint" />
      </div>

      <!-- 中间：请求配置 -->
      <div class="main-panel">
        <ApiRequestPanel :selected-endpoint="selectedEndpoint" v-model:path-params="pathParams"
          v-model:query-params="queryParams" v-model:request-body="requestBody"
          v-model:custom-headers="customHeaders" :full-url="fullUrl" :request-headers="requestHeaders"
          :loading="loading" @send="sendRequest" />
      </div>

      <!-- 右侧：响应结果 -->
      <div class="right-panel">
        <ApiResponsePanel :result="lastResult" />
      </div>

      <!-- 底部：历史记录 -->
      <div class="bottom-panel">
        <ApiHistoryPanel :history="history" @select="loadFromHistory" @clear="clearHistory" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Terminal } from 'lucide-vue-next'
import { useApiTest } from '@/composables/test'
import {
  ApiEndpointSelect,
  ApiRequestPanel,
  ApiResponsePanel,
  ApiHistoryPanel
} from '@/components/test'

/**
 * API 测试页面
 *
 * 提供可视化的 API 测试界面，包括：
 * - 端点选择器（按分类展示所有 API）
 * - 请求配置面板（参数、请求体、请求头）
 * - 响应结果面板（状态码、响应头、响应体）
 * - 历史记录面板（最近 20 条请求）
 */

const {
  endpoints,
  groupedEndpoints,
  selectedEndpoint,
  pathParams,
  queryParams,
  requestBody,
  customHeaders,
  history,
  lastResult,
  loading,
  fullUrl,
  requestHeaders,
  selectEndpoint,
  sendRequest,
  clearHistory,
  loadFromHistory
} = useApiTest()
</script>

<style scoped>
.api-test-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 16px;
  overflow: hidden;
}

.page-header {
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 4px;
}

.header-title h1 {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.header-title svg {
  color: var(--primary-color);
}

.header-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
  margin-left: 36px;
}

.page-content {
  flex: 1;
  display: grid;
  grid-template-columns: 280px 1fr 1fr;
  grid-template-rows: 1fr 200px;
  gap: 16px;
  min-height: 0;
}

.sidebar {
  grid-row: 1 / 3;
  min-height: 0;
  overflow: hidden;
}

.main-panel {
  min-height: 0;
  overflow: hidden;
}

.right-panel {
  min-height: 0;
  overflow: hidden;
}

.bottom-panel {
  grid-column: 2 / 4;
  min-height: 0;
  overflow: hidden;
}

/* 确保子组件可以滚动 */
.sidebar > *,
.main-panel > *,
.right-panel > *,
.bottom-panel > * {
  height: 100%;
}

/* 平板端适配 */
@media (max-width: 1024px) {
  .page-content {
    grid-template-columns: 240px 1fr;
    grid-template-rows: 1fr 1fr 200px;
  }

  .sidebar {
    grid-row: 1 / 4;
  }

  .right-panel {
    grid-column: 2;
  }

  .bottom-panel {
    grid-column: 2;
  }
}

/* 移动端适配 */
@media (max-width: 768px) {
  .api-test-page {
    padding: 12px;
    gap: 12px;
  }

  .page-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
  }

  .sidebar,
  .main-panel,
  .right-panel,
  .bottom-panel {
    min-height: 400px;
    flex-shrink: 0;
  }

  .header-title h1 {
    font-size: 18px;
  }

  .header-desc {
    margin-left: 0;
    margin-top: 4px;
  }
}
</style>
