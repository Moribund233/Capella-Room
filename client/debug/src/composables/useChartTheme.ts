import { computed } from 'vue'
import { useThemeStore } from '@/store'

/**
 * 图表主题色板
 * 与项目 CSS 变量保持一致，确保视觉统一
 */
const chartColorPalette = [
  '#5b8ff9',
  '#5ad8a6',
  '#5d7092',
  '#f6bd16',
  '#e8684a',
  '#6dc8ec',
  '#9270ca',
  '#ff9d4d',
  '#269a99',
  '#ff99c3',
]

/**
 * 图表基础配置对象结构
 */
export interface ChartBaseOption {
  color: string[]
  backgroundColor: string
  textStyle: { fontFamily: string }
  title: { textStyle: { color: string }; subtextStyle: { color: string } }
  legend: { textStyle: { color: string }; pageTextStyle: { color: string } }
  tooltip: {
    backgroundColor: string
    borderColor: string
    textStyle: { color: string }
    extraCssText: string
  }
  categoryAxis: {
    axisLine: { lineStyle: { color: string } }
    axisTick: { lineStyle: { color: string } }
    axisLabel: { color: string }
    splitLine: { lineStyle: { color: string } }
  }
  valueAxis: {
    axisLine: { show: boolean }
    axisTick: { show: boolean }
    axisLabel: { color: string }
    splitLine: { lineStyle: { color: string } }
  }
  radar: {
    axisLine: { lineStyle: { color: string } }
    splitLine: { lineStyle: { color: string } }
    splitArea: { areaStyle: { color: string[] } }
    axisName: { color: string }
  }
}

/**
 * 亮色主题图表基础配置
 */
const lightBaseOption: ChartBaseOption = {
  color: chartColorPalette,
  backgroundColor: 'transparent',
  textStyle: {
    fontFamily: '"Inter", "Helvetica Neue", "PingFang SC", "Microsoft YaHei", sans-serif',
  },
  title: {
    textStyle: { color: 'rgba(0, 0, 0, 0.85)' },
    subtextStyle: { color: 'rgba(0, 0, 0, 0.45)' },
  },
  legend: {
    textStyle: { color: 'rgba(0, 0, 0, 0.65)' },
    pageTextStyle: { color: 'rgba(0, 0, 0, 0.65)' },
  },
  tooltip: {
    backgroundColor: '#ffffff',
    borderColor: '#e8e8e8',
    textStyle: { color: 'rgba(0, 0, 0, 0.85)' },
    extraCssText: 'box-shadow: 0 3px 6px -4px rgba(0,0,0,0.08), 0 6px 16px 0 rgba(0,0,0,0.05);',
  },
  categoryAxis: {
    axisLine: { lineStyle: { color: '#e8e8e8' } },
    axisTick: { lineStyle: { color: '#e8e8e8' } },
    axisLabel: { color: 'rgba(0, 0, 0, 0.45)' },
    splitLine: { lineStyle: { color: '#f0f0f0' } },
  },
  valueAxis: {
    axisLine: { show: false },
    axisTick: { show: false },
    axisLabel: { color: 'rgba(0, 0, 0, 0.45)' },
    splitLine: { lineStyle: { color: '#f0f0f0' } },
  },
  radar: {
    axisLine: { lineStyle: { color: '#e8e8e8' } },
    splitLine: { lineStyle: { color: '#f0f0f0' } },
    splitArea: {
      areaStyle: {
        color: ['rgba(250,249,246,0.5)', 'rgba(250,249,246,0.3)'],
      },
    },
    axisName: { color: 'rgba(0, 0, 0, 0.65)' },
  },
}

/**
 * 暗色主题图表基础配置
 */
const darkBaseOption: ChartBaseOption = {
  color: chartColorPalette,
  backgroundColor: 'transparent',
  textStyle: {
    fontFamily: '"Inter", "Helvetica Neue", "PingFang SC", "Microsoft YaHei", sans-serif',
  },
  title: {
    textStyle: { color: 'rgba(255, 255, 255, 0.85)' },
    subtextStyle: { color: 'rgba(255, 255, 255, 0.45)' },
  },
  legend: {
    textStyle: { color: 'rgba(255, 255, 255, 0.65)' },
    pageTextStyle: { color: 'rgba(255, 255, 255, 0.65)' },
  },
  tooltip: {
    backgroundColor: '#1f1f1f',
    borderColor: '#434343',
    textStyle: { color: 'rgba(255, 255, 255, 0.85)' },
    extraCssText: 'box-shadow: 0 3px 6px -4px rgba(0,0,0,0.32), 0 6px 16px 0 rgba(0,0,0,0.24);',
  },
  categoryAxis: {
    axisLine: { lineStyle: { color: '#434343' } },
    axisTick: { lineStyle: { color: '#434343' } },
    axisLabel: { color: 'rgba(255, 255, 255, 0.45)' },
    splitLine: { lineStyle: { color: 'rgba(255,255,255,0.06)' } },
  },
  valueAxis: {
    axisLine: { show: false },
    axisTick: { show: false },
    axisLabel: { color: 'rgba(255, 255, 255, 0.45)' },
    splitLine: { lineStyle: { color: 'rgba(255,255,255,0.06)' } },
  },
  radar: {
    axisLine: { lineStyle: { color: '#434343' } },
    splitLine: { lineStyle: { color: 'rgba(255,255,255,0.06)' } },
    splitArea: {
      areaStyle: {
        color: ['rgba(255,255,255,0.02)', 'rgba(255,255,255,0.04)'],
      },
    },
    axisName: { color: 'rgba(255, 255, 255, 0.65)' },
  },
}

/**
 * 图表主题配置
 */
export interface ChartTheme {
  /** 是否暗色主题 */
  isDark: boolean
  /** 当前主题的基础配置对象 */
  baseOption: ChartBaseOption
  /** 图表色板 */
  colorPalette: string[]
}

/**
 * 使用图表主题
 * 提供与项目主题系统联动的 ECharts 主题配置
 *
 * @returns 图表主题配置
 *
 * @example
 * ```ts
 * const { isDark, baseOption, colorPalette } = useChartTheme()
 *
 * // 在 ECharts 配置中合并基础主题
 * const option = {
 *   ...baseOption,
 *   series: [...]
 * }
 * ```
 */
export function useChartTheme(): ChartTheme {
  const themeStore = useThemeStore()

  const isDark = computed(() => themeStore.isDark)

  const baseOption = computed<ChartBaseOption>(() =>
    isDark.value ? darkBaseOption : lightBaseOption,
  )

  return {
    isDark: isDark.value,
    baseOption: baseOption.value,
    colorPalette: chartColorPalette,
  }
}
