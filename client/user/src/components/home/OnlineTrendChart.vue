<template>
  <ChartCard title="在线用户趋势" subtitle="近24小时活跃用户变化" :loading="loading">
    <LineChart
      :x-axis="xAxisData"
      :series="seriesData"
      :show-zoom="false"
      :show-grid="true"
      y-axis-name="用户数"
    />
  </ChartCard>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ChartCard from '@/components/common/charts/ChartCard.vue'
import LineChart from '@/components/common/charts/LineChart.vue'
import type { LineSeries } from '@/components/common/charts/LineChart.vue'

const loading = ref(true)

// 生成近24小时的时间标签
const generateTimeLabels = () => {
  const labels: string[] = []
  const now = new Date()
  for (let i = 23; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 60 * 60 * 1000)
    labels.push(time.getHours().toString().padStart(2, '0') + ':00')
  }
  return labels
}

// 生成模拟数据
const generateData = (min: number, max: number) => {
  return Array.from({ length: 24 }, () => Math.floor(Math.random() * (max - min + 1)) + min)
}

const xAxisData = ref<string[]>([])
const seriesData = ref<LineSeries[]>([])

onMounted(() => {
  // 模拟加载数据
  setTimeout(() => {
    xAxisData.value = generateTimeLabels()
    seriesData.value = [
      {
        name: '在线用户',
        data: generateData(50, 150),
        smooth: true,
        area: true,
      },
      {
        name: '活跃房间',
        data: generateData(5, 20),
        smooth: true,
        area: false,
      },
    ]
    loading.value = false
  }, 500)
})
</script>
