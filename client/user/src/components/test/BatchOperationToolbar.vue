<template>
  <div class="batch-toolbar">
    <!-- 第一行：创建和导入/导出 -->
    <div class="toolbar-row">
      <!-- 批量创建区域 -->
      <div class="create-section">
        <div class="create-input-group">
          <n-input-number
            v-model:value="createCount"
            :min="1"
            :max="50"
            :disabled="loading"
            class="count-input"
          />
          <n-button
            type="primary"
            :loading="loading && currentOperation?.includes('创建')"
            :disabled="loading"
            class="create-btn"
            @click="handleCreate"
          >
            <template #icon>
              <n-icon :component="UserPlus" />
            </template>
            <span class="btn-text">创建</span>
          </n-button>
        </div>
      </div>

      <!-- 导入/导出区域 -->
      <div class="io-section">
        <n-button
          size="small"
          :disabled="loading || userStats.total === 0"
          @click="handleExport"
        >
          <template #icon>
            <n-icon :component="Download" />
          </template>
          <span class="btn-label">导出</span>
        </n-button>

        <n-upload
          accept=".json"
          :show-file-list="false"
          :disabled="loading"
          @before-upload="handleImport"
        >
          <n-button size="small" :disabled="loading">
            <template #icon>
              <n-icon :component="Upload" />
            </template>
            <span class="btn-label">导入</span>
          </n-button>
        </n-upload>
      </div>
    </div>

    <!-- 第二行：操作按钮 -->
    <div class="action-row">
      <div class="action-grid">
        <n-button
          size="small"
          :disabled="loading || userStats.offline === 0"
          @click="$emit('login-users')"
        >
          <template #icon>
            <n-icon :component="LogIn" />
          </template>
          <span class="btn-label">登录</span>
          <span v-if="userStats.offline > 0" class="badge">{{ userStats.offline }}</span>
        </n-button>

        <n-button
          size="small"
          :disabled="loading || userStats.online === 0"
          @click="$emit('connect-web-socket')"
        >
          <template #icon>
            <n-icon :component="Wifi" />
          </template>
          <span class="btn-label">连WS</span>
        </n-button>

        <n-button
          size="small"
          :disabled="loading || userStats.connected === 0"
          @click="$emit('disconnect-web-socket')"
        >
          <template #icon>
            <n-icon :component="WifiOff" />
          </template>
          <span class="btn-label">断WS</span>
        </n-button>

        <n-button
          size="small"
          :disabled="loading || userStats.online === 0"
          @click="$emit('refresh-tokens')"
        >
          <template #icon>
            <n-icon :component="RefreshCw" />
          </template>
          <span class="btn-label">刷新</span>
        </n-button>

        <n-button
          size="small"
          :disabled="loading || userStats.online === 0"
          @click="$emit('logout-users')"
        >
          <template #icon>
            <n-icon :component="LogOut" />
          </template>
          <span class="btn-label">登出</span>
        </n-button>

        <n-button
          size="small"
          type="error"
          ghost
          :disabled="loading || userStats.total === 0"
          @click="$emit('clear-all')"
        >
          <template #icon>
            <n-icon :component="Trash2" />
          </template>
          <span class="btn-label">清空</span>
        </n-button>
      </div>
    </div>

    <!-- 进度显示 -->
    <div v-if="operationProgress" class="progress-section">
      <div class="progress-header">
        <span class="operation-name">{{ currentOperation }}</span>
        <span class="progress-text">{{ operationProgress.current }} / {{ operationProgress.total }}</span>
      </div>
      <n-progress
        :percentage="Math.round((operationProgress.current / operationProgress.total) * 100)"
        :show-indicator="false"
        type="line"
        :processing="true"
        class="progress-bar"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NButton, NIcon, NInputNumber, NProgress, NUpload, useMessage } from 'naive-ui'
import type { UploadFileInfo } from 'naive-ui'
import {
  UserPlus,
  LogIn,
  LogOut,
  Wifi,
  WifiOff,
  RefreshCw,
  Trash2,
  Download,
  Upload,
} from 'lucide-vue-next'
import type { ExportedUserCredential } from '@/store/testUsers'

defineProps<{
  loading: boolean
  currentOperation: string
  operationProgress: { current: number; total: number } | null
  userStats: {
    total: number
    online: number
    connected: number
    offline: number
  }
}>()

const emit = defineEmits<{
  'create-users': [count: number]
  'login-users': []
  'connect-web-socket': []
  'disconnect-web-socket': []
  'refresh-tokens': []
  'logout-users': []
  'clear-all': []
  'export-credentials': []
  'import-credentials': [credentials: ExportedUserCredential[]]
}>()

const message = useMessage()
const createCount = ref(5)

function handleCreate() {
  emit('create-users', createCount.value)
}

function handleExport() {
  emit('export-credentials')
}

async function handleImport(data: { file: UploadFileInfo, fileList: UploadFileInfo[] }) {
  const file = data.file.file
  if (!file) {
    message.error('请选择文件')
    return false
  }

  try {
    const text = await file.text()
    const credentials = JSON.parse(text) as ExportedUserCredential[]

    if (!Array.isArray(credentials)) {
      message.error('文件格式错误：应为数组')
      return false
    }

    if (credentials.length === 0) {
      message.error('文件为空')
      return false
    }

    emit('import-credentials', credentials)
    return false
  } catch {
    message.error('文件解析失败，请检查 JSON 格式')
    return false
  }
}
</script>

<style scoped>
.batch-toolbar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  background: var(--bg-container);
  border-radius: 12px;
}

/* 工具栏行 */
.toolbar-row {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 创建区域 */
.create-section {
  display: flex;
  justify-content: center;
}

.create-input-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.count-input {
  width: 80px;
}

.create-btn {
  flex-shrink: 0;
}

/* 导入/导出区域 */
.io-section {
  display: flex;
  gap: 8px;
  justify-content: center;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.io-section :deep(.n-button) {
  flex: 1;
  justify-content: center;
}

.io-section :deep(.n-upload) {
  flex: 1;
}

.io-section :deep(.n-upload-trigger) {
  width: 100%;
}

/* 操作按钮行 */
.action-row {
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

/* 操作按钮网格 */
.action-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.action-grid :deep(.n-button) {
  justify-content: center;
  padding: 8px 4px;
  height: auto;
  flex-direction: column;
  gap: 4px;
}

.action-grid :deep(.n-button__icon) {
  margin: 0;
  font-size: 16px;
}

.btn-label {
  font-size: 11px;
  line-height: 1.2;
}

.badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  background: var(--primary-color);
  color: white;
  font-size: 10px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 进度显示 */
.progress-section {
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
  font-size: 12px;
}

.operation-name {
  color: var(--text-secondary);
}

.progress-text {
  color: var(--primary-color);
  font-weight: 500;
}

.progress-bar {
  margin: 0;
}

.progress-bar :deep(.n-progress-line) {
  height: 4px;
}

/* 桌面端适配 */
@media (min-width: 768px) {
  .batch-toolbar {
    padding: 16px;
  }

  .toolbar-row {
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
  }

  .create-section {
    justify-content: flex-start;
  }

  .io-section {
    padding-top: 0;
    border-top: none;
    border-left: 1px solid var(--border-color);
    padding-left: 16px;
    justify-content: flex-end;
  }

  .io-section :deep(.n-button) {
    flex: none;
  }

  .io-section :deep(.n-upload) {
    flex: none;
  }

  .action-row {
    padding-top: 16px;
  }

  .action-grid {
    grid-template-columns: repeat(6, auto);
    justify-content: flex-start;
    gap: 12px;
  }

  .action-grid :deep(.n-button) {
    flex-direction: row;
    padding: 6px 12px;
    height: 32px;
  }

  .action-grid :deep(.n-button__icon) {
    margin-right: 4px;
    font-size: 14px;
  }

  .btn-label {
    font-size: 12px;
  }

  .progress-section {
    padding-top: 12px;
  }
}
</style>
