# 页面开发指南

## 概述

在 CapellaUI 中开发页面遵循**配置驱动**的理念。标准流程包括：

1. 创建页面组件
2. 配置路由
3. 配置侧边栏菜单
4. 可选：配置 Dock 栏

## 标准页面结构

### 简单页面

适用于内容单一的页面：

```vue
<!-- src/views/SimpleView.vue -->
<template>
  <div class="simple-view">
    <h1>页面标题</h1>
    <div class="content">
      <!-- 页面内容 -->
    </div>
  </div>
</template>

<script setup lang="ts">
// 页面逻辑
</script>

<style scoped>
.simple-view {
  padding: 24px;
}
</style>
```

### 带子路由的页面

适用于需要标签页切换的页面：

```vue
<!-- src/views/ComplexView.vue -->
<template>
  <div class="complex-view">
    <h1>复杂页面</h1>
    <!-- 子路由视图 -->
    <RouterView />
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style scoped>
.complex-view {
  padding: 24px;
  min-height: 100%;
}
</style>
```

## 完整示例：数据管理页面

### 步骤1：创建页面组件

```vue
<!-- src/views/DataView.vue -->
<template>
  <div class="data-view">
    <h1>数据管理</h1>
    <p class="description">管理您的数据资源</p>

    <!-- 子路由视图 -->
    <div class="content-area">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style scoped>
.data-view {
  padding: 24px;
  min-height: 100%;
}

.data-view h1 {
  margin-bottom: 8px;
  color: var(--text-primary);
}

.description {
  color: var(--text-secondary);
  margin-bottom: 24px;
}

.content-area {
  background: var(--bg-container);
  border-radius: 12px;
  padding: 24px;
  min-height: 400px;
}
</style>
```

### 步骤2：创建子页面组件

子页面组件存放在 `src/pages/` 目录下，按功能模块组织：

```vue
<!-- src/pages/data/ListPanel.vue -->
<template>
  <div class="list-panel">
    <h2>数据列表</h2>
    <n-data-table :columns="columns" :data="data" />
  </div>
</template>

<script setup lang="ts">
import { NDataTable } from 'naive-ui'

const columns = [
  { title: 'ID', key: 'id' },
  { title: '名称', key: 'name' },
  { title: '状态', key: 'status' },
]

const data = [
  { id: 1, name: '数据1', status: '正常' },
  { id: 2, name: '数据2', status: '异常' },
]
</script>
```

```vue
<!-- src/pages/data/SettingsPanel.vue -->
<template>
  <div class="settings-panel">
    <h2>数据设置</h2>
    <n-form>
      <n-form-item label="默认分页大小">
        <n-input-number v-model:value="pageSize" />
      </n-form-item>
    </n-form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NForm, NFormItem, NInputNumber } from 'naive-ui'

const pageSize = ref(20)
</script>
```

### 步骤3：配置路由

编辑 `src/router/routes.ts`：

```typescript
{
  path: 'data',
  name: 'Data',
  component: () => import('@/views/DataView.vue'),
  redirect: '/data/list',  // 默认重定向到列表页
  meta: {
    title: '数据管理',
    requiresAuth: true,
  },
  children: [
    {
      path: 'list',
      name: 'DataList',
      component: () => import('@/pages/data/ListPanel.vue'),
      meta: { title: '数据列表' },
    },
    {
      path: 'settings',
      name: 'DataSettings',
      component: () => import('@/pages/data/SettingsPanel.vue'),
      meta: { title: '数据设置' },
    },
  ],
}
```

### 步骤4：配置侧边栏菜单

编辑 `src/config/ui.ts`：

```typescript
export const sidebarConfig = {
  items: [
    // ... 其他菜单
    {
      name: '数据管理',
      icon: 'Database', // Lucide 图标
      path: '/data',
    },
  ],
}
```

### 步骤5：配置 Dock 栏（可选）

编辑 `src/config/ui.ts`：

```typescript
export const dockConfig = {
  // ... 其他页面配置
  data: {
    enabled: true,
    position: 'bottom',
    offset: 24,
    items: [
      {
        key: 'list',
        label: '列表',
        icon: 'List',
        path: '/data/list',
      },
      {
        key: 'settings',
        label: '设置',
        icon: 'Settings',
        path: '/data/settings',
      },
    ],
  },
}
```

## StatusBar 状态栏使用

StatusBar 组件用于在页面底部显示单行状态信息，支持动态内容，移动端自动滚动显示。

### 基本用法

```vue
<script setup lang="ts">
import { h, onUnmounted } from 'vue'
import { useStatusBar } from '@/composables'

const { setContent, clearContent } = useStatusBar()

// 设置简单文本
setContent('系统状态: 正常运行')

// 设置带链接的内容
setContent(['发现新版本 ', h('a', { href: '#', onClick: handleUpdate }, '立即更新')])

// 页面卸载时清除
onUnmounted(() => {
  clearContent()
})

function handleUpdate() {
  // 处理更新逻辑
}
</script>
```

### 在页面中使用

```vue
<!-- src/views/DashboardView.vue -->
<template>
  <div class="dashboard-view">
    <h1>仪表盘</h1>
    <!-- 页面内容 -->
  </div>
</template>

<script setup lang="ts">
import { h, onMounted, onUnmounted } from 'vue'
import { useStatusBar } from '@/composables'

const { setContent, clearContent } = useStatusBar()

onMounted(() => {
  // 设置状态栏内容
  setContent([
    h('span', { class: 'status-indicator' }, [h('span', { class: 'status-dot' }), ' 系统正常运行']),
    ' | ',
    h('a', { href: '#' }, '查看日志'),
  ])
})

onUnmounted(() => {
  // 清除状态栏内容
  clearContent()
})
</script>
```

### StatusBar 支持的样式类

```vue
<!-- 状态指示器 -->
<span class="status-indicator">
  <span class="status-dot"></span> 正常
</span>

<!-- 警告状态 -->
<span class="status-indicator">
  <span class="status-dot warning"></span> 警告
</span>

<!-- 错误状态 -->
<span class="status-indicator">
  <span class="status-dot error"></span> 错误
</span>
```

### 注意事项

1. **页面级管理**：每个页面应该独立管理自己的 StatusBar 内容
2. **及时清除**：在页面卸载时调用 `clearContent()` 清除内容
3. **内容类型**：支持字符串和 VNode，不支持组件类型
4. **自动滚动**：当内容宽度超过容器时，移动端会自动滚动显示

## ECharts 图表组件使用

CapellaUI 提供了基于 ECharts 的图表组件，支持自动主题适配。

### 导入图表组件

```vue
<script setup lang="ts">
import { ChartCard, LineChart, BarChart, PieChart, AreaChart } from '@/components/common/charts'
</script>
```

### ChartCard 图表卡片

ChartCard 是图表的容器组件，提供标题、加载状态、空状态等功能。

```vue
<template>
  <chart-card
    title="访问趋势"
    subtitle="近7日访问量变化"
    :loading="loading"
    :empty="isEmpty"
    empty-text="暂无数据"
    min-height="300px"
  >
    <!-- 图表内容 -->
    <line-chart :x-axis="xAxis" :series="series" />
  </chart-card>
</template>
```

**ChartCard 属性：**

| 属性      | 类型                           | 默认值     | 说明             |
| --------- | ------------------------------ | ---------- | ---------------- |
| title     | string                         | ''         | 卡片标题         |
| subtitle  | string                         | ''         | 副标题           |
| bordered  | boolean                        | false      | 是否显示边框     |
| segmented | boolean                        | false      | 是否显示分段线   |
| loading   | boolean                        | false      | 是否加载中       |
| spinSize  | 'small' \| 'medium' \| 'large' | 'medium'   | 加载指示器大小   |
| empty     | boolean                        | false      | 是否数据为空     |
| emptyText | string                         | '暂无数据' | 空状态提示文本   |
| minHeight | string \| number               | '280px'    | 内容区域最小高度 |

### LineChart 折线图

```vue
<template>
  <line-chart
    :x-axis="['周一', '周二', '周三', '周四', '周五', '周六', '周日']"
    :series="[
      { name: '访问量', data: [120, 200, 150, 80, 70, 110, 130], smooth: true },
      { name: '用户数', data: [60, 140, 100, 40, 50, 80, 90], smooth: true, area: true },
    ]"
    title="访问趋势"
    y-axis-name="次数"
    :show-legend="true"
    :show-tooltip="true"
  />
</template>
```

**LineChart 属性：**

| 属性           | 类型              | 默认值 | 说明           |
| -------------- | ----------------- | ------ | -------------- |
| xAxis          | string[]          | 必填   | X 轴数据       |
| series         | LineSeries[]      | 必填   | 系列数据       |
| title          | string            | ''     | 图表标题       |
| showLegend     | boolean           | true   | 是否显示图例   |
| legendPosition | 'top' \| 'bottom' | 'top'  | 图例位置       |
| showTooltip    | boolean           | true   | 是否显示提示框 |
| showZoom       | boolean           | false  | 是否显示缩放   |
| yAxisName      | string            | ''     | Y 轴名称       |
| showGrid       | boolean           | true   | 是否显示网格线 |
| loading        | boolean           | false  | 是否加载中     |

**LineSeries 类型：**

```typescript
interface LineSeries {
  name: string // 系列名称
  data: number[] // 数据数组
  smooth?: boolean // 是否平滑曲线
  area?: boolean // 是否显示面积填充
  lineStyle?: object // 线条样式
  markPoint?: object // 标记点配置
  markLine?: object // 标记线配置
}
```

### BarChart 柱状图

```vue
<template>
  <bar-chart
    :x-axis="['产品A', '产品B', '产品C', '产品D']"
    :series="[
      { name: '销量', data: [120, 200, 150, 80], barWidth: '40%' },
      { name: '库存', data: [60, 140, 100, 40] },
    ]"
    title="产品销售统计"
    :horizontal="false"
  />
</template>
```

**BarChart 属性：**

| 属性        | 类型        | 默认值 | 说明           |
| ----------- | ----------- | ------ | -------------- |
| xAxis       | string[]    | 必填   | X 轴数据       |
| series      | BarSeries[] | 必填   | 系列数据       |
| title       | string      | ''     | 图表标题       |
| horizontal  | boolean     | false  | 是否横向展示   |
| showLegend  | boolean     | true   | 是否显示图例   |
| showTooltip | boolean     | true   | 是否显示提示框 |
| yAxisName   | string      | ''     | Y 轴名称       |

**BarSeries 类型：**

```typescript
interface BarSeries {
  name: string                    // 系列名称
  data: number[]                 // 数据数组
  barWidth?: string \| number     // 柱子宽度
  barBorderRadius?: number       // 柱子圆角
  showBackground?: boolean       // 是否显示背景
  backgroundStyle?: object       // 背景样式
  label?: object                 // 标签配置
  itemStyle?: object             // 数据项样式
}
```

### PieChart 饼图

```vue
<template>
  <pie-chart
    :data="[
      { name: '直接访问', value: 335 },
      { name: '邮件营销', value: 310 },
      { name: '联盟广告', value: 234 },
      { name: '视频广告', value: 135 },
      { name: '搜索引擎', value: 1548 },
    ]"
    type="doughnut"
    title="访问来源"
    :show-label="true"
    legend-position="bottom"
  />
</template>
```

**PieChart 属性：**

| 属性            | 类型                                   | 默认值   | 说明           |
| --------------- | -------------------------------------- | -------- | -------------- |
| data            | PieDataItem[]                          | 必填     | 饼图数据       |
| type            | 'pie' \| 'doughnut' \| 'rose'          | 'pie'    | 饼图类型       |
| title           | string                                 | ''       | 图表标题       |
| showLegend      | boolean                                | true     | 是否显示图例   |
| legendPosition  | 'top' \| 'bottom' \| 'left' \| 'right' | 'bottom' | 图例位置       |
| showTooltip     | boolean                                | true     | 是否显示提示框 |
| showLabel       | boolean                                | true     | 是否显示标签   |
| labelLineLength | number                                 | 15       | 标签引导线长度 |

**PieDataItem 类型：**

```typescript
interface PieDataItem {
  name: string // 数据名称
  value: number // 数据值
  itemStyle?: object // 自定义样式
  label?: object // 自定义标签
}
```

### AreaChart 面积图

```vue
<template>
  <area-chart
    :x-axis="['1月', '2月', '3月', '4月', '5月', '6月']"
    :series="[
      { name: '收入', data: [820, 932, 901, 934, 1290, 1330], areaOpacity: 0.3 },
      { name: '支出', data: [620, 732, 701, 734, 1090, 1130], areaOpacity: 0.3 },
    ]"
    title="财务收支"
    :stack="true"
    y-axis-name="金额（万元）"
  />
</template>
```

**AreaChart 属性：**

| 属性        | 类型         | 默认值 | 说明           |
| ----------- | ------------ | ------ | -------------- |
| xAxis       | string[]     | 必填   | X 轴数据       |
| series      | AreaSeries[] | 必填   | 系列数据       |
| title       | string       | ''     | 图表标题       |
| stack       | boolean      | false  | 是否堆叠       |
| showLegend  | boolean      | true   | 是否显示图例   |
| showTooltip | boolean      | true   | 是否显示提示框 |
| yAxisName   | string       | ''     | Y 轴名称       |

**AreaSeries 类型：**

```typescript
interface AreaSeries {
  name: string // 系列名称
  data: number[] // 数据数组
  smooth?: boolean // 是否平滑曲线
  areaOpacity?: number // 填充透明度
  lineStyle?: object // 线条样式
  markPoint?: object // 标记点配置
  markLine?: object // 标记线配置
}
```

### 完整图表示例

```vue
<!-- src/views/StatisticsView.vue -->
<template>
  <div class="statistics-view">
    <h1>数据统计</h1>

    <div class="charts-grid">
      <!-- 折线图 -->
      <chart-card title="访问趋势" subtitle="近7日数据">
        <line-chart :x-axis="weekDays" :series="visitSeries" :loading="loading" />
      </chart-card>

      <!-- 柱状图 -->
      <chart-card title="产品销售">
        <bar-chart :x-axis="products" :series="salesSeries" />
      </chart-card>

      <!-- 饼图 -->
      <chart-card title="访问来源">
        <pie-chart :data="sourceData" type="doughnut" />
      </chart-card>

      <!-- 面积图 -->
      <chart-card title="财务收支">
        <area-chart :x-axis="months" :series="financeSeries" :stack="true" />
      </chart-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ChartCard, LineChart, BarChart, PieChart, AreaChart } from '@/components/common/charts'

const loading = ref(true)
const weekDays = ref(['周一', '周二', '周三', '周四', '周五', '周六', '周日'])
const visitSeries = ref([{ name: '访问量', data: [120, 200, 150, 80, 70, 110, 130], smooth: true }])

const products = ref(['产品A', '产品B', '产品C', '产品D'])
const salesSeries = ref([{ name: '销量', data: [120, 200, 150, 80] }])

const sourceData = ref([
  { name: '直接访问', value: 335 },
  { name: '邮件营销', value: 310 },
  { name: '联盟广告', value: 234 },
  { name: '视频广告', value: 135 },
  { name: '搜索引擎', value: 1548 },
])

const months = ref(['1月', '2月', '3月', '4月', '5月', '6月'])
const financeSeries = ref([
  { name: '收入', data: [820, 932, 901, 934, 1290, 1330] },
  { name: '支出', data: [620, 732, 701, 734, 1090, 1130] },
])

onMounted(() => {
  // 模拟数据加载
  setTimeout(() => {
    loading.value = false
  }, 1000)
})
</script>

<style scoped>
.statistics-view {
  padding: 24px;
}

.charts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
  margin-top: 24px;
}

@media (max-width: 768px) {
  .charts-grid {
    grid-template-columns: 1fr;
  }
}
</style>
```

## 页面开发最佳实践

### 1. 组件组织

```
src/
├── views/              # 页面级组件（路由入口）
│   └── DataView.vue    # 数据管理主页面
├── pages/              # 子页面面板（路由子组件）
│   └── data/
│       ├── ListPanel.vue
│       └── SettingsPanel.vue
├── components/         # 可复用组件
│   ├── common/         # 通用组件
│   │   ├── DockBar.vue
│   │   ├── QuickBar.vue
│   │   ├── IconPicker.vue
│   │   └── charts/     # 图表组件
│   │       ├── LineChart.vue
│   │       ├── BarChart.vue
│   │       ├── PieChart.vue
│   │       └── AreaChart.vue
│   └── layout/         # 布局组件
│       ├── MainLayout.vue
│       ├── AppHeader.vue
│       └── AppSideBar.vue
```

### 2. 样式规范

```vue
<style scoped>
/* 使用 CSS 变量 */
.data-view {
  padding: 24px;
  background: var(--bg-container);
  color: var(--text-primary);
}

/* 响应式适配 */
@media (max-width: 768px) {
  .data-view {
    padding: 16px;
  }
}
</style>
```

### 3. 状态管理

```typescript
// 使用 Pinia Store
import { useDataStore } from '@/store/data'

const dataStore = useDataStore()

// 读取状态
const list = computed(() => dataStore.list)

// 调用 action
onMounted(() => {
  dataStore.fetchList()
})
```

### 4. 图表使用建议

- **自动主题适配**：图表组件会自动适配当前主题，无需手动配置
- **响应式**：所有图表组件默认启用 `autoresize`，会自动适应容器大小
- **加载状态**：使用 `loading` 属性显示加载状态
- **空状态**：使用 ChartCard 的 `empty` 属性处理无数据情况

### 5. StatusBar 使用建议

- **页面级管理**：每个页面独立管理自己的 StatusBar 内容
- **及时清理**：页面卸载时务必调用 `clearContent()`
- **内容简洁**：StatusBar 适合显示简短的状态信息，不宜过长
- **链接支持**：可以使用 VNode 添加链接，实现交互功能
