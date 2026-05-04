# 页面开发指南

## 概述

在 SeredeliUI 中开发页面遵循**配置驱动**的理念。标准流程包括：

1. 创建页面组件
2. 配置路由
3. 配置侧边栏菜单
4. 配置 Dock 栏

## 标准页面结构

### 目录结构

- **简单单页**：直接编辑主视图（`/views`）
- **复杂单页**：主视图（`/views`）+ 组件模式（`/components`）
- **带子路由**：主视图（`/views`）+ 子页面（`/pages`）（复杂子页面再使用组件（`/components`））


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
dataStore.fetchList()
```

### 4. 路由跳转

```typescript
import { useRouter } from 'vue-router'

const router = useRouter()

// 编程式导航
router.push('/data/list')
router.push({ name: 'DataSettings' })
```

## 常用模式

### 带加载状态的页面

```vue
<template>
  <div class="page">
    <n-spin :show="loading">
      <div v-if="!loading">
        <!-- 内容 -->
      </div>
    </n-spin>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NSpin } from 'naive-ui'

const loading = ref(true)

onMounted(async () => {
  await fetchData()
  loading.value = false
})
</script>
```

### 带错误处理的页面

```vue
<script setup lang="ts">
import { ref } from 'vue'

const error = ref<string | null>(null)

async function fetchData() {
  try {
    error.value = null
    // 请求数据
  } catch (e) {
    error.value = '加载失败，请重试'
  }
}
</script>
```

## 调试技巧

### 查看当前路由

```typescript
import { useRoute } from 'vue-router'

const route = useRoute()
console.log('当前路由:', route.path)
console.log('路由参数:', route.params)
console.log('查询参数:', route.query)
```

### 查看 UI 配置

```typescript
import { useUIStore } from '@/store'

const uiStore = useUIStore()
console.log('合并配置:', uiStore.mergedConfig)
console.log('侧边栏:', uiStore.sidebarConfig)
console.log('Dock:', uiStore.dockConfig)
```

## 图表组件使用

SeredeliUI 封装了基于 ECharts 的图表组件，位于 `src/components/common/charts/`，支持自动主题适配。

### 支持的图表类型

| 组件      | 说明   | 导入路径                     |
| --------- | ------ | ---------------------------- |
| LineChart | 折线图 | `@/components/common/charts` |
| BarChart  | 柱状图 | `@/components/common/charts` |
| PieChart  | 饼图   | `@/components/common/charts` |
| AreaChart | 面积图 | `@/components/common/charts` |

### 折线图示例

```vue
<template>
  <n-card title="访问趋势">
    <line-chart :x-axis="weekDays" :series="visitSeries" :show-legend="true" :show-grid="true" />
  </n-card>
</template>

<script setup lang="ts">
import { LineChart } from '@/components/common/charts'

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
/* 图表需要固定高度容器 */
.n-card :deep(.line-chart) {
  height: 300px;
}
</style>
```

### 柱状图示例

```vue
<template>
  <n-card title="用户来源">
    <bar-chart :x-axis="sourceLabels" :series="sourceSeries" :show-legend="false" />
  </n-card>
</template>

<script setup lang="ts">
import { BarChart } from '@/components/common/charts'

const sourceLabels = ['直接访问', '邮件营销', '联盟广告', '视频广告', '搜索引擎']
const sourceSeries = [
  {
    name: '用户数',
    data: [320, 302, 301, 334, 390],
  },
]
</script>
```

### 饼图示例

```vue
<template>
  <n-card title="设备分布">
    <pie-chart :data="deviceData" :show-legend="true" :radius="['40%', '70%']" />
  </n-card>
</template>

<script setup lang="ts">
import { PieChart } from '@/components/common/charts'

const deviceData = [
  { name: '桌面端', value: 1048 },
  { name: '移动端', value: 735 },
  { name: '平板', value: 580 },
  { name: '其他', value: 300 },
]
</script>
```

### 面积图示例

```vue
<template>
  <n-card title="销售趋势">
    <area-chart :labels="months" :series="salesSeries" />
  </n-card>
</template>

<script setup lang="ts">
import { AreaChart } from '@/components/common/charts'

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
</script>
```

### 图表通用属性

| 属性           | 类型       | 默认值  | 说明         |
| -------------- | ---------- | ------- | ------------ |
| xAxis / labels | `string[]` | -       | X 轴数据     |
| series         | `Series[]` | -       | 系列数据     |
| title          | `string`   | `''`    | 图表标题     |
| showLegend     | `boolean`  | `true`  | 显示图例     |
| showTooltip    | `boolean`  | `true`  | 显示提示框   |
| showGrid       | `boolean`  | `true`  | 显示网格线   |
| autoresize     | `boolean`  | `true`  | 自动调整大小 |
| loading        | `boolean`  | `false` | 加载状态     |

### 主题适配

图表组件自动跟随系统明暗主题切换，无需额外配置。主题颜色通过 CSS 变量和 `useChartTheme` 组合式函数统一管理。

## 下一步

- [配置系统](./configuration.md) - 深入了解配置系统
- [QuickBar 开发](./quickbar-development.md) - 添加自定义快捷按钮
- [接入真实后端](./backend-integration.md) - 连接真实 API
