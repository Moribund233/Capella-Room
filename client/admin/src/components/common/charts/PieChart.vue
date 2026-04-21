<template>
  <v-chart class="pie-chart" :option="chartOption" :autoresize="autoresize" :loading="loading"
    :loading-options="loadingOptions" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { PieChart as EChartsPieChart } from 'echarts/charts'
import {
  TooltipComponent,
  LegendComponent,
  TitleComponent,
} from 'echarts/components'
import VChart from 'vue-echarts'
import { useChartTheme } from '@/composables/useChartTheme'
import type { EChartsOption } from 'echarts'
import type { ChartBaseOption } from '@/composables/useChartTheme'

use([
  CanvasRenderer,
  EChartsPieChart,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
])

/**
 * 饼图数据项
 */
export interface PieDataItem {
  /** 数据名称 */
  name: string
  /** 数据值 */
  value: number
  /** 自定义样式 */
  itemStyle?: Record<string, unknown>
  /** 自定义标签 */
  label?: Record<string, unknown>
}

/**
 * 饼图类型
 */
export type PieType = 'pie' | 'doughnut' | 'rose'

/**
 * 饼图组件属性
 */
interface Props {
  /** 饼图数据 */
  data: PieDataItem[]
  /** 图表标题 */
  title?: string
  /** 饼图类型 */
  type?: PieType
  /** 是否显示图例 */
  showLegend?: boolean
  /** 图例位置 */
  legendPosition?: 'top' | 'bottom' | 'left' | 'right'
  /** 是否显示提示框 */
  showTooltip?: boolean
  /** 是否显示标签 */
  showLabel?: boolean
  /** 标签引导线长度 */
  labelLineLength?: number
  /** 是否自动调整大小 */
  autoresize?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** 额外的 ECharts 配置 */
  option?: EChartsOption
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  type: 'pie',
  showLegend: true,
  legendPosition: 'bottom',
  showTooltip: true,
  showLabel: true,
  labelLineLength: 15,
  autoresize: true,
  loading: false,
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
 * 图例位置映射
 */
const legendPositionMap: Record<string, Record<string, string | number>> = {
  top: { top: props.title ? 28 : 0, left: 'center' },
  bottom: { bottom: 0, left: 'center' },
  left: { left: 0, top: 'center', orient: 'vertical' as const },
  right: { right: 0, top: 'center', orient: 'vertical' as const },
}

/**
 * 饼图半径配置
 */
const radiusConfig = computed(() => {
  switch (props.type) {
    case 'doughnut':
      return ['40%', '70%']
    case 'rose':
      return ['20%', '70%']
    default:
      return '65%'
  }
})

/**
 * 构建 ECharts 配置
 */
const chartOption = computed<EChartsOption>(() => {
  const themeBase: ChartBaseOption = baseOption

  const legendConfig = legendPositionMap[props.legendPosition]

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
        type: 'scroll',
        ...legendConfig,
        ...themeBase.legend,
      }
      : { show: false },
    tooltip: props.showTooltip
      ? {
        trigger: 'item',
        formatter: '{b}: {c} ({d}%)',
        ...themeBase.tooltip,
      }
      : { show: false },
    series: [
      {
        type: 'pie',
        radius: radiusConfig.value,
        center: ['50%', '50%'],
        roseType: props.type === 'rose' ? 'area' : undefined,
        data: props.data,
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.2)',
          },
        },
        label: props.showLabel
          ? {
            show: true,
            formatter: '{b}\n{d}%',
            ...themeBase.textStyle,
          }
          : { show: false },
        labelLine: props.showLabel
          ? {
            show: true,
            length: props.labelLineLength,
            length2: 10,
          }
          : { show: false },
        animationType: 'scale',
        animationEasing: 'elasticOut',
        animationDelay: () => Math.random() * 200,
      },
    ],
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
.pie-chart {
  width: 100%;
  height: 100%;
  min-height: 280px;
}
</style>
