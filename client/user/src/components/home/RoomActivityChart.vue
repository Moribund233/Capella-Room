<template>
  <ChartCard title="房间类型分布" subtitle="各类型房间占比统计" :loading="loading">
    <PieChart
      :data="pieData"
      type="doughnut"
      :show-legend="true"
      legend-position="bottom"
      :show-label="true"
    />
  </ChartCard>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ChartCard from '@/components/common/charts/ChartCard.vue'
import PieChart from '@/components/common/charts/PieChart.vue'
import type { PieDataItem } from '@/components/common/charts/PieChart.vue'

const loading = ref(true)

const pieData = ref<PieDataItem[]>([])

onMounted(() => {
  // 模拟加载数据
  setTimeout(() => {
    pieData.value = [
      { name: '技术交流', value: 35 },
      { name: '闲聊灌水', value: 28 },
      { name: '游戏开黑', value: 20 },
      { name: '学习讨论', value: 12 },
      { name: '其他', value: 5 },
    ]
    loading.value = false
  }, 600)
})
</script>
