<template>
  <v-chart class="bar-chart" :option="chartOption" :autoresize="autoresize" :loading="loading"
    :loading-options="loadingOptions" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart as EChartsBarChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  DataZoomComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'
import { useChartTheme } from '@/composables/useChartTheme'
import type { EChartsOption } from 'echarts'
import type { ChartBaseOption } from '@/composables/useChartTheme'

use([
  CanvasRenderer,
  EChartsBarChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  DataZoomComponent,
])

/**
 * 柱状图系列数据项
 */
export interface BarSeries {
  /** 系列名称 */
  name: string
  /** 数据数组 */
  data: number[]
  /** 柱子宽度 */
  barWidth?: string | number
  /** 柱子圆角 */
  barBorderRadius?: number | number[]
  /** 是否显示背景 */
  showBackground?: boolean
  /** 背景样式 */
  backgroundStyle?: Record<string, unknown>
  /** 标签配置 */
  label?: Record<string, unknown>
  /** 数据项样式 */
  itemStyle?: Record<string, unknown>
}

/**
 * 柱状图组件属性
 */
interface Props {
  /** X 轴数据 */
  xAxis: string[]
  /** 系列数据 */
  series: BarSeries[]
  /** 图表标题 */
  title?: string
  /** 是否横向展示 */
  horizontal?: boolean
  /** 是否显示图例 */
  showLegend?: boolean
  /** 图例位置 */
  legendPosition?: 'top' | 'bottom'
  /** 是否显示提示框 */
  showTooltip?: boolean
  /** 是否显示缩放 */
  showZoom?: boolean
  /** 是否自动调整大小 */
  autoresize?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** Y 轴名称 */
  yAxisName?: string
  /** 是否显示网格线 */
  showGrid?: boolean
  /** 额外的 ECharts 配置 */
  option?: EChartsOption
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  horizontal: false,
  showLegend: true,
  legendPosition: 'top',
  showTooltip: true,
  showZoom: false,
  autoresize: true,
  loading: false,
  yAxisName: '',
  showGrid: true,
  option: () => ({}),
})

const { baseOption } = useChartTheme()

/**
 * 加载状态配置
 */
const loadingOptions = {
  text: '加载中...',
  color: '#5b8ff9',
  textColor: 'var(--text-secondary)',
  maskColor: 'rgba(255, 255, 255, 0.6)',
}

/**
 * 构建 ECharts 配置
 */
const chartOption = computed<EChartsOption>(() => {
  const themeBase: ChartBaseOption = baseOption

  const seriesConfig = props.series.map((s) => ({
    name: s.name,
    type: 'bar' as const,
    data: s.data,
    barWidth: s.barWidth ?? (props.series.length > 1 ? '40%' : '50%'),
    itemStyle: {
      borderRadius: s.barBorderRadius ?? (props.horizontal ? [0, 4, 4, 0] : [4, 4, 0, 0]),
      ...s.itemStyle,
    },
    showBackground: s.showBackground ?? false,
    backgroundStyle: s.backgroundStyle,
    label: s.label,
  }))

  const categoryAxis = {
    type: 'category' as const,
    data: props.xAxis,
    ...themeBase.categoryAxis,
  }

  const valueAxis = {
    type: 'value' as const,
    name: props.yAxisName,
    ...themeBase.valueAxis,
    splitLine: {
      show: props.showGrid,
      ...themeBase.valueAxis.splitLine,
    },
  }

  const option: EChartsOption = {
    ...themeBase,
    title: props.title
      ? {
        text: props.title,
        left: 'center',
        top: 0,
        ...themeBase.title,
      }
      : undefined,
    legend: props.showLegend
      ? {
        show: true,
        top: props.title ? 28 : 0,
        ...themeBase.legend,
      }
      : { show: false },
    tooltip: props.showTooltip
      ? {
        trigger: 'axis',
        axisPointer: {
          type: 'shadow',
          shadowStyle: {
            opacity: 0.1,
          },
        },
        ...themeBase.tooltip,
      }
      : { show: false },
    grid: {
      left: '3%',
      right: '4%',
      bottom: props.showZoom ? '12%' : '3%',
      top: props.title ? 72 : props.showLegend ? 40 : 16,
      containLabel: true,
    },
    xAxis: props.horizontal ? valueAxis : categoryAxis,
    yAxis: props.horizontal ? categoryAxis : valueAxis,
    dataZoom: props.showZoom
      ? [
        {
          type: 'inside',
          start: 0,
          end: 100,
        },
        {
          start: 0,
          end: 100,
          height: 20,
          bottom: 0,
        },
      ]
      : undefined,
    series: seriesConfig,
  }

  return mergeOption(option, props.option)
})

/**
 * 合并配置对象
 * @param target 目标配置
 * @param source 源配置
 * @returns 合并后的配置
 */
function mergeOption(target: EChartsOption, source: EChartsOption): EChartsOption {
  if (!source || Object.keys(source).length === 0) {
    return target
  }

  const result = { ...target }

  for (const key in source) {
    if (Object.prototype.hasOwnProperty.call(source, key)) {
      const sourceValue = source[key as keyof EChartsOption]
      const targetValue = result[key as keyof EChartsOption]

      if (
        typeof sourceValue === 'object' &&
        sourceValue !== null &&
        !Array.isArray(sourceValue) &&
        typeof targetValue === 'object' &&
        targetValue !== null &&
        !Array.isArray(targetValue)
      ) {
        result[key as keyof EChartsOption] = mergeOption(
          targetValue as EChartsOption,
          sourceValue as EChartsOption,
        ) as never
      } else {
        result[key as keyof EChartsOption] = sourceValue as never
      }
    }
  }

  return result
}
</script>

<style scoped>
.bar-chart {
  width: 100%;
  height: 100%;
  min-height: 280px;
}
</style>
