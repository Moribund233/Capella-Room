<template>
  <div class="home-view">
    <!-- 统计卡片区域 -->
    <div class="stats-grid">
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="在线用户" :value="onlineUsers">
          <template #prefix>
            <Users class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="总用户数" :value="totalUsers">
          <template #prefix>
            <UserCheck class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="活跃房间" :value="activeRooms">
          <template #prefix>
            <MessageSquare class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="今日消息" :value="todayMessages">
          <template #prefix>
            <Mail class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
    </div>

    <!-- 图表区域 -->
    <div class="charts-grid">
      <!-- 折线图：用户在线趋势 -->
      <chart-card title="在线用户趋势" subtitle="近24小时在线用户数变化">
        <line-chart :x-axis="hours" :series="onlineTrendSeries" y-axis-name="在线用户数" />
      </chart-card>

      <!-- 柱状图：消息统计 -->
      <chart-card title="消息统计" subtitle="近7日消息发送量">
        <bar-chart :x-axis="weekDays" :series="messageSeries" y-axis-name="消息数" />
      </chart-card>

      <!-- 面积图：用户增长 -->
      <chart-card title="用户增长趋势" subtitle="近6个月用户注册量">
        <area-chart :x-axis="months" :series="userGrowthSeries" y-axis-name="新增用户数" />
      </chart-card>

      <!-- 饼图：房间类型分布 -->
      <chart-card title="房间类型分布" subtitle="各类房间占比">
        <pie-chart :data="roomTypeData" type="doughnut" />
      </chart-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NCard, NStatistic } from 'naive-ui'
import { Users, UserCheck, MessageSquare, Mail } from 'lucide-vue-next'
import { ChartCard, LineChart, BarChart, AreaChart, PieChart } from '@/components/common/charts'

/**
 * 统计数据（模拟数据，后续对接后端API）
 */
const onlineUsers = 128
const totalUsers = 3456
const activeRooms = 24
const todayMessages = 8934

/**
 * 折线图数据：24小时在线用户趋势
 */
const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`)
const onlineTrendSeries = [
  {
    name: '在线用户',
    data: [45, 38, 32, 28, 25, 30, 45, 68, 92, 105, 118, 125, 128, 122, 115, 108, 112, 125, 138, 142, 135, 120, 95, 65],
    smooth: true,
  },
]

/**
 * 柱状图数据：近7日消息统计
 */
const weekDays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日']
const messageSeries = [
  {
    name: '消息数',
    data: [7234, 8123, 7567, 8934, 9876, 11234, 10567],
  },
]

/**
 * 面积图数据：近6个月用户增长
 */
const months = ['1月', '2月', '3月', '4月', '5月', '6月']
const userGrowthSeries = [
  {
    name: '新增用户',
    data: [320, 452, 389, 534, 678, 823],
    areaOpacity: 0.3,
  },
]

/**
 * 饼图数据：房间类型分布
 */
const roomTypeData = [
  { name: '公开房间', value: 45 },
  { name: '私密房间', value: 28 },
  { name: '临时房间', value: 15 },
  { name: '系统房间', value: 8 },
]
</script>

<style scoped>
.home-view {
  min-height: 100%;
  padding: 24px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  transition: var(--transition-base);
}

.stat-card:hover {
  transform: translateY(-2px);
}

.stat-icon {
  color: var(--color-primary);
  margin-right: 8px;
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .charts-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }

  .home-view {
    padding: 16px;
  }
}
</style>
