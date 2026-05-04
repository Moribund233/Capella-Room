<template>
  <v-chart class="line-chart" :option="chartOption" :autoresize="autoresize" :loading="loading"
    :loading-options="loadingOptions" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart as EChartsLineChart } from 'echarts/charts'
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
  EChartsLineChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  DataZoomComponent,
])

/**
 * 折线数据项
 */
export interface LineSeries {
  /** 系列名称 */
  name: string
  /** 数据数组 */
  data: number[]
  /** 是否平滑曲线 */
  smooth?: boolean
  /** 是否显示面积填充 */
  area?: boolean
  /** 线条样式 */
  lineStyle?: Record<string, unknown>
  /** 标记点配置 */
  markPoint?: Record<string, unknown>
  /** 标记线配置 */
  markLine?: Record<string, unknown>
}

/**
 * 折线图组件属性
 */
interface Props {
  /** X 轴数据 */
  xAxis: string[]
  /** 系列数据 */
  series: LineSeries[]
  /** 图表标题 */
  title?: string
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
    type: 'line' as const,
    data: s.data,
    smooth: s.smooth ?? true,
    symbol: 'circle' as const,
    symbolSize: 6,
    lineStyle: {
      width: 2,
      ...s.lineStyle,
    },
    itemStyle: {
      borderWidth: 2,
    },
    areaStyle: s.area
      ? {
        opacity: 0.1,
      }
      : undefined,
    markPoint: s.markPoint,
    markLine: s.markLine,
  }))

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
          type: 'line',
          lineStyle: {
            width: 1,
            type: 'dashed',
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
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: props.xAxis,
      ...themeBase.categoryAxis,
    },
    yAxis: {
      type: 'value' as const,
      name: props.yAxisName,
      ...themeBase.valueAxis,
      splitLine: {
        show: props.showGrid,
        ...themeBase.valueAxis.splitLine,
      },
    },
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

  // 合并用户自定义配置
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
.line-chart {
  width: 100%;
  height: 100%;
  min-height: 280px;
}
</style>
