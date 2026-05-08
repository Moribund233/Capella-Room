<template>
  <div class="system-logs-view">
    <n-card title="系统日志流" class="logs-card">
      <template #header-extra>
        <n-space>
          <n-tag type="info">WebSocket 实时推送</n-tag>
          <n-tooltip>
            <template #trigger>
              <n-icon size="18" class="help-icon">
                <help-circle />
              </n-icon>
            </template>
            <div class="help-content">
              <p>系统日志流功能说明：</p>
              <ul>
                <li>实时接收服务端日志推送</li>
                <li>支持按日志级别和模块过滤</li>
                <li>仅管理员可订阅系统日志</li>
                <li>日志最多保留1000条</li>
              </ul>
            </div>
          </n-tooltip>
        </n-space>
      </template>

      <LogStreamPanel />
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NSpace, NTag, NTooltip, NIcon } from 'naive-ui'
import { HelpCircle } from 'lucide-vue-next'
import { LogStreamPanel } from '@/components/system-logs'
</script>

<style scoped lang="scss">
.system-logs-view {
  padding: 24px;
  height: 100%;
  box-sizing: border-box;
}

.logs-card {
  height: 100%;
  display: flex;
  flex-direction: column;

  :deep(.n-card__content) {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 0;
    overflow: hidden;
  }
}

.help-icon {
  cursor: help;
  color: var(--text-color-secondary);
}

.help-content {
  max-width: 300px;

  p {
    margin: 0 0 8px;
    font-weight: 500;
  }

  ul {
    margin: 0;
    padding-left: 16px;

    li {
      margin-bottom: 4px;
    }
  }
}
</style>
