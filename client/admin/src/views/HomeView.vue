<template>
  <div class="home-view">
    <!-- 统计卡片区域 -->
    <div class="stats-grid">
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="总访问量" :value="12345">
          <template #prefix>
            <TrendingUp class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="活跃用户" :value="890">
          <template #prefix>
            <Users class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="订单数" :value="456">
          <template #prefix>
            <ShoppingCart class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
      <n-card class="stat-card" :bordered="false">
        <n-statistic label="转化率" :value="12.5" suffix="%">
          <template #prefix>
            <Percent class="stat-icon" :size="20" />
          </template>
        </n-statistic>
      </n-card>
    </div>

    <!-- 图表区域 -->
    <div class="charts-grid">
      <!-- 折线图：访问趋势 -->
      <chart-card title="访问趋势" subtitle="近7日访问量变化">
        <line-chart :x-axis="weekDays" :series="visitSeries" y-axis-name="访问量" />
      </chart-card>

      <!-- 柱状图：用户来源 -->
      <chart-card title="用户来源" subtitle="各渠道用户分布">
        <bar-chart :x-axis="sourceLabels" :series="sourceSeries" y-axis-name="用户数" />
      </chart-card>

      <!-- 面积图：销售趋势 -->
      <chart-card title="销售趋势" subtitle="近6个月销售额对比">
        <area-chart :x-axis="months" :series="salesSeries" y-axis-name="销售额（万元）" />
      </chart-card>

      <!-- 饼图：设备分布 -->
      <chart-card title="设备分布" subtitle="用户访问设备占比">
        <pie-chart :data="deviceData" type="doughnut" />
      </chart-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NCard, NStatistic } from 'naive-ui'
import { TrendingUp, Users, ShoppingCart, Percent } from 'lucide-vue-next'
import { ChartCard, LineChart, BarChart, AreaChart, PieChart } from '@/components/common/charts'

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

/**
 * 柱状图数据
 */
const sourceLabels = ['直接访问', '邮件营销', '联盟广告', '视频广告', '搜索引擎']
const sourceSeries = [
  {
    name: '用户数',
    data: [320, 302, 301, 334, 390],
  },
]

/**
 * 面积图数据
 */
const months = ['1月', '2月', '3月', '4月', '5月', '6月']
const salesSeries = [
  {
    name: '线上销售',
    data: [120, 132, 101, 134, 90, 230],
    areaOpacity: 0.3,
  },
  {
    name: '线下销售',
    data: [220, 182, 191, 234, 290, 330],
    areaOpacity: 0.3,
  },
]

/**
 * 饼图数据
 */
const deviceData = [
  { name: '桌面端', value: 1048 },
  { name: '移动端', value: 735 },
  { name: '平板', value: 580 },
  { name: '其他', value: 300 },
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
