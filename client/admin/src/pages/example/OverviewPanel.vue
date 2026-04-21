<template>
  <div class="overview-panel">
    <h2>概览</h2>
    <p>这是概览面板的内容</p>

    <!-- 统计卡片区域 -->
    <div class="stats-grid">
      <div class="stat-card">
        <n-statistic label="总访问量" :value="12345" />
      </div>
      <div class="stat-card">
        <n-statistic label="活跃用户" :value="890" />
      </div>
      <div class="stat-card">
        <n-statistic label="订单数" :value="456" />
      </div>
      <div class="stat-card">
        <n-statistic label="转化率" :value="12.5" suffix="%" />
      </div>
    </div>

    <!-- 图表区域 -->
    <n-card title="访问趋势" class="chart-card">
      <div class="chart-container">
        <line-chart :x-axis="weekDays" :series="visitSeries" :show-legend="true" :show-grid="true" />
      </div>
    </n-card>

    <!-- 数据表格 - 支持水平滚动 -->
    <n-card title="最近活动" class="table-card">
      <div class="table-scroll-wrapper">
        <n-data-table :columns="columns" :data="tableData" :pagination="pagination" :scroll-x="600" />
      </div>
    </n-card>

    <!-- 信息列表 -->
    <n-card title="系统通知" class="list-card">
      <n-list>
        <n-list-item v-for="(item, index) in notifications" :key="index">
          <n-thing :title="item.title" :description="item.content">
            <template #avatar>
              <n-avatar :style="{ background: item.color }">
                <n-icon>
                  <component :is="item.icon" />
                </n-icon>
              </n-avatar>
            </template>
          </n-thing>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- 描述列表 -->
    <n-card title="系统信息" class="desc-card">
      <n-descriptions bordered :columns="2">
        <n-descriptions-item label="系统名称">SeredeliUI</n-descriptions-item>
        <n-descriptions-item label="版本号">v1.0.0</n-descriptions-item>
        <n-descriptions-item label="运行环境">Production</n-descriptions-item>
        <n-descriptions-item label="最后更新">2026-04-17</n-descriptions-item>
        <n-descriptions-item label="服务器状态" :span="2">
          <n-tag type="success">正常运行</n-tag>
        </n-descriptions-item>
      </n-descriptions>
    </n-card>

    <!-- 时间线 -->
    <n-card title="更新日志" class="timeline-card">
      <n-timeline>
        <n-timeline-item type="success" title="版本发布" content="v1.0.0 正式发布" time="2026-04-17" />
        <n-timeline-item type="info" title="功能更新" content="新增 DockBar 组件" time="2026-04-16" />
        <n-timeline-item type="warning" title="问题修复" content="修复主题切换问题" time="2026-04-15" />
        <n-timeline-item title="项目启动" content="SeredeliUI 项目启动" time="2026-04-01" />
      </n-timeline>
    </n-card>

    <!-- 折叠面板 -->
    <n-card title="常见问题" class="collapse-card">
      <n-collapse>
        <n-collapse-item title="如何切换主题？" name="1">
          <div>点击右上角的主题切换按钮即可在明暗主题间切换。</div>
        </n-collapse-item>
        <n-collapse-item title="如何配置 DockBar？" name="2">
          <div>在 ui.ts 配置文件中配置 dock 属性，支持页面级配置。</div>
        </n-collapse-item>
        <n-collapse-item title="支持哪些图标？" name="3">
          <div>目前支持 Lucide 图标库的所有图标。</div>
        </n-collapse-item>
      </n-collapse>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { h } from 'vue'
import {
  NStatistic,
  NCard,
  NDataTable,
  NList,
  NListItem,
  NThing,
  NAvatar,
  NIcon,
  NDescriptions,
  NDescriptionsItem,
  NTag,
  NTimeline,
  NTimelineItem,
  NCollapse,
  NCollapseItem,
} from 'naive-ui'
import { Bell, CheckCircle, Info, AlertCircle } from 'lucide-vue-next'
import { LineChart } from '@/components/common/charts'

const columns = [
  { title: 'ID', key: 'id', width: 60 },
  { title: '用户', key: 'user', width: 100 },
  { title: '操作', key: 'action', width: 120 },
  { title: '时间', key: 'time', width: 160 },
  {
    title: '状态',
    key: 'status',
    width: 80,
    render(row: { status: string }) {
      return h('n-tag', { type: row.status === '成功' ? 'success' : 'error' }, { default: () => row.status })
    },
  },
]

const tableData = [
  { id: 1, user: '张三', action: '登录', time: '2026-04-17 10:00', status: '成功' },
  { id: 2, user: '李四', action: '修改配置', time: '2026-04-17 09:30', status: '成功' },
  { id: 3, user: '王五', action: '删除数据', time: '2026-04-17 09:00', status: '失败' },
  { id: 4, user: '赵六', action: '导出报表', time: '2026-04-17 08:30', status: '成功' },
  { id: 5, user: '钱七', action: '登录', time: '2026-04-17 08:00', status: '成功' },
]

const pagination = {
  pageSize: 5,
}

const notifications = [
  { title: '系统更新', content: '系统将于今晚进行例行维护', color: '#2080f0', icon: Info },
  { title: '任务完成', content: '数据备份任务已成功完成', color: '#18a058', icon: CheckCircle },
  { title: '安全提醒', content: '检测到异常登录尝试', color: '#d03050', icon: AlertCircle },
  { title: '新消息', content: '您有3条未读消息', color: '#f0a020', icon: Bell },
]

/**
 * 折线图数据
 */
const weekDays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日']
const visitSeries = [
  {
    name: '本周',
    data: [820, 932, 901, 934, 1290, 1330, 1320],
    smooth: true,
  },
  {
    name: '上周',
    data: [720, 832, 801, 834, 1190, 1230, 1220],
    smooth: true,
  },
]
</script>

<style scoped>
.overview-panel {
  display: flex;
  flex-direction: column;
  padding: 16px;
  min-height: 0;
}

.overview-panel h2 {
  margin-bottom: 8px;
  color: var(--text-primary);
  flex-shrink: 0;
}

.overview-panel>p {
  margin-bottom: 16px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

/* 统计卡片网格 - 响应式 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
  flex-shrink: 0;
}

.stat-card {
  padding: 20px;
  background: var(--bg-base);
  border-radius: 8px;
  border: 1px solid var(--border-color-base);
}

/* 卡片通用样式 */
.chart-card,
.table-card,
.list-card,
.desc-card,
.timeline-card,
.collapse-card {
  margin-bottom: 24px;
  flex-shrink: 0;
}

/* 图表容器 */
.chart-container {
  height: 300px;
}

/* 表格滚动容器 */
.table-scroll-wrapper {
  width: 100%;
  overflow-x: auto;
  overflow-y: hidden;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .overview-panel {
    padding: 12px;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .stat-card {
    padding: 16px;
  }

  .chart-card,
  .table-card,
  .list-card,
  .desc-card,
  .timeline-card,
  .collapse-card {
    margin-bottom: 16px;
  }
}

@media (max-width: 480px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
}

/* 确保卡片内容不溢出 */
:deep(.n-card) {
  max-width: 100%;
}

/* 表格容器样式 */
:deep(.n-data-table) {
  min-width: 100%;
}
</style>
