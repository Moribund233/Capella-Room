# 快速开始

## 环境准备

### 系统要求

- Node.js >= 18.0.0
- npm >= 9.0.0 或 yarn >= 1.22.0
- 现代浏览器（Chrome、Firefox、Edge、Safari）

### 安装依赖

```bash
# 克隆项目
git clone <your-repo-url>
cd SeredeliUI

# 安装依赖
npm install

# 启动开发服务器
npm run dev
```

访问 http://localhost:3000 查看应用。

## 框架结构速览

### 5分钟了解核心概念

#### 1. 配置驱动（Config-Driven）

框架的核心思想：**通过配置定义UI，而非修改组件代码**。

```typescript
// src/config/ui.ts
export const sidebarConfig = {
  items: [
    { name: '首页', icon: 'LayoutDashboard', path: '/home' },
    { name: '示例', icon: 'FileText', path: '/example' },
  ]
}
```

修改这个配置，侧边栏会自动更新。

#### 2. 三层架构

```
配置层 → 状态层 → 呈现层
```

- **配置层**：`src/config/ui.ts` 定义UI结构
- **状态层**：`src/store/ui.ts` 管理运行时状态
- **呈现层**：`src/components/` 组件渲染界面

#### 3. 事件通信

组件间通过事件总线通信：

```typescript
// 触发事件
import { executeQuickAction } from '@/composables'
executeQuickAction('user', 'profile')

// 监听事件
import { quickActionBus } from '@/composables'
quickActionBus.on('quick-action', (event) => {
  console.log(event.key, event.childKey)
})
```

## 第一个页面

### 步骤1：创建页面组件

创建 `src/views/MyPageView.vue`：

```vue
<template>
  <div class="my-page">
    <h1>我的页面</h1>
    <p>这是使用 SeredeliUI 创建的第一个页面</p>
  </div>
</template>

<script setup lang="ts">
// 页面逻辑
</script>

<style scoped>
.my-page {
  padding: 24px;
}
</style>
```

### 步骤2：配置路由

编辑 `src/router/routes.ts`：

```typescript
{
  path: 'my-page',
  name: 'MyPage',
  component: () => import('@/views/MyPageView.vue'),
  meta: {
    title: '我的页面',
    requiresAuth: true,
  },
}
```

### 步骤3：配置侧边栏菜单

编辑 `src/config/ui.ts`：

```typescript
export const sidebarConfig = {
  items: [
    // ... 其他菜单
    {
      name: '我的页面',
      icon: 'FileText',  // Lucide 图标名
      path: '/my-page',
    },
  ]
}
```

### 步骤4：查看效果

刷新页面，侧边栏会出现"我的页面"菜单，点击即可访问。

## 常用操作速查

### 添加侧边栏菜单

```typescript
// src/config/ui.ts
{
  name: '菜单名称',
  icon: 'IconName',  // 从 lucide-vue-next 选择
  path: '/route-path',
}
```

### 添加 Dock 栏

```typescript
// src/config/ui.ts
export const dockConfig = {
  myPage: {
    enabled: true,
    position: 'bottom',
    items: [
      { key: 'tab1', label: '标签1', icon: 'Icon1', path: '/my-page/tab1' },
      { key: 'tab2', label: '标签2', icon: 'Icon2', path: '/my-page/tab2' },
    ]
  }
}
```

### 添加 Quick 按钮

```typescript
// src/config/ui.ts
export const quickBarConfig = [
  // ... 其他按钮
  {
    key: 'myAction',
    display: 'visible',  // 或 'dropdown'
    type: 'action',      // 或 'menu'
    icon: 'MyIcon',
    label: '我的操作',
  }
]
```

监听事件：

```typescript
// 在任意组件中
import { quickActionBus } from '@/composables'

quickActionBus.on('quick-action', (event) => {
  if (event.key === 'myAction') {
    console.log('我的操作被点击了')
  }
})
```

## 图标使用

框架使用 [Lucide Vue](https://lucide.dev/icons/) 图标库。

### 查找图标

1. 访问 https://lucide.dev/icons/
2. 选择需要的图标
3. 复制图标名称（如 `Settings`、`User`、`Home`）

### 使用图标

```typescript
// 配置中使用
{ icon: 'Settings', label: '设置' }

// 组件中使用
import { Settings } from 'lucide-vue-next'
```

## 下一步

- [页面开发指南](./page-development.md) - 学习开发复杂页面
- [配置系统](./configuration.md) - 深入了解配置系统
- [QuickBar 开发](./quickbar-development.md) - 自定义快捷按钮
