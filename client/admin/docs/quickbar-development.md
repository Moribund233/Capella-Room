# QuickBar 开发指南

## 概述

QuickBar 是位于页面顶部的快捷按钮栏，用于放置全局可访问的功能按钮，如主题切换、布局控制、用户中心等。

## 架构设计

SeredeliUI 的 Quick 系统采用**工厂模式 + 组合式函数**的架构：

```
┌─────────────────────────────────────────────────────────────┐
│                     QuickBar.vue                            │
│              （工厂分发器，只负责匹配和渲染）                   │
├─────────────────────────────────────────────────────────────┤
│  useQuickTheme │ useQuickLayout │ useQuickUser │ 自定义...   │
│  （每个 Quick 按钮独立的组合式函数，包含完整业务逻辑）           │
├─────────────────────────────────────────────────────────────┤
│  AboutModal.vue │ LoginModalContent.vue │ 其他弹窗组件...    │
│  （放在 components/quick/ 目录下，由对应组合式函数调用）        │
├─────────────────────────────────────────────────────────────┤
│              MainLayout.vue # GlobalModal                   │
│              （统一的全局弹窗渲染出口）                        │
└─────────────────────────────────────────────────────────────┘
```

**核心原则**：

- 每个 Quick 按钮必须有对应的组合式函数
- 业务逻辑封装在组合式函数内部
- 弹窗组件放在 `components/quick/` 目录
- 所有弹窗通过 `useGlobalModal` 统一渲染

## 内置 Quick 按钮

框架内置了以下 Quick 按钮：

| key     | 功能       | 类型   | 组合式函数     | 说明                      |
| ------- | ---------- | ------ | -------------- | ------------------------- |
| sidebar | 切换侧边栏 | action | useQuickLayout | 点击展开/收起侧边栏       |
| footer  | 切换底部栏 | action | useQuickLayout | 点击显示/隐藏底部栏       |
| theme   | 主题切换   | menu   | useQuickTheme  | 下拉菜单选择浅色/深色主题 |
| user    | 用户中心   | menu   | useQuickUser   | 下拉菜单提供用户相关操作  |

内置按钮无需额外开发，配置即可使用。

## 添加自定义 Quick 按钮

以"通知中心"为例，演示完整的开发流程：

### 步骤1：创建弹窗组件

```vue
<!-- src/components/quick/NotificationPanel.vue -->
<template>
  <div class="notification-panel">
    <n-list>
      <n-list-item v-for="item in notifications" :key="item.id">
        <n-thing :title="item.title" :description="item.content" />
      </n-list-item>
    </n-list>
  </div>
</template>

<script setup lang="ts">
import { NList, NListItem, NThing } from 'naive-ui'

const notifications = [
  { id: 1, title: '系统通知', content: '欢迎使用 SeredeliUI' },
  { id: 2, title: '更新提醒', content: '新版本已发布' },
]
</script>

<style scoped>
.notification-panel {
  padding: 16px;
}
</style>
```

### 步骤2：创建组合式函数

```typescript
// src/composables/quick/useQuickNotification.ts
import { computed, ref } from 'vue'
import { useGlobalModal } from '@/composables/useGlobalModal'
import NotificationPanel from '@/components/quick/NotificationPanel.vue'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

export function useQuickNotification(
  config: QuickConfigItem,
  context: QuickContext,
): UseQuickReturn {
  const { open } = useGlobalModal()

  // 未读消息数
  const unreadCount = ref(3)

  // 是否有未读（用于徽标显示）
  const isActive = computed(() => unreadCount.value > 0)

  /**
   * 显示通知面板
   */
  function showNotificationPanel(): void {
    open({
      title: '通知中心',
      component: NotificationPanel,
      componentProps: {},
      preset: 'card',
      width: 480,
      closable: true,
      maskClosable: true,
    })

    // 打开后清空未读
    unreadCount.value = 0
  }

  /**
   * 点击处理
   */
  function onClick(): void {
    showNotificationPanel()
  }

  /**
   * 子菜单选择（如果有）
   */
  function onSelect(childKey: string): void {
    if (childKey === 'mark-all-read') {
      unreadCount.value = 0
    } else {
      context.emitAction(config.key, childKey)
    }
  }

  return {
    isActive,
    currentIcon: computed(() => config.icon),
    onClick,
    onSelect: config.type === 'menu' ? onSelect : undefined,
  }
}
```

### 步骤3：注册到工厂

编辑 `src/components/common/QuickBar.vue`：

```typescript
import { useQuickNotification } from '@/composables/quick/useQuickNotification'

const quickFactories: Record<string, typeof useQuickTheme> = {
  theme: useQuickTheme,
  sidebar: useQuickLayout,
  footer: useQuickLayout,
  user: useQuickUser,
  notification: useQuickNotification, // 注册新 Quick
}
```

### 步骤4：配置 UI

编辑 `src/config/ui.ts`：

```typescript
export const quickBarConfig: QuickItemConfig[] = [
  // ... 其他按钮
  {
    key: 'notification', // 必须匹配工厂注册的 key
    display: 'visible',
    type: 'action',
    icon: 'Bell',
    label: '通知中心',
    badge: 3, // 显示未读数
  },
]
```

## 配置说明

### display 属性

| 值       | 说明                       |
| -------- | -------------------------- |
| visible  | 按钮直接显示在 QuickBar 上 |
| dropdown | 按钮聚合在"更多"下拉菜单中 |

### type 属性

| 值     | 说明             |
| ------ | ---------------- |
| action | 点击直接执行动作 |
| menu   | 点击显示下拉菜单 |

### 图标选择

从 [Lucide Icons](https://lucide.dev/icons/) 选择图标名称：

```typescript
// 常用图标
{
  icon: 'Bell'
} // 通知
{
  icon: 'Settings'
} // 设置
{
  icon: 'Search'
} // 搜索
{
  icon: 'Plus'
} // 添加
{
  icon: 'Download'
} // 下载
{
  icon: 'Share'
} // 分享
```

## 组合式函数 API

### UseQuickReturn 接口

```typescript
interface UseQuickReturn {
  /** 当前是否激活（用于图标切换） */
  isActive: Ref<boolean>
  /** 当前显示的图标（用于状态切换） */
  currentIcon: Ref<string>
  /** 点击主按钮的处理函数 */
  onClick: () => void
  /** 选择子菜单项的处理函数（可选，仅 menu 类型需要） */
  onSelect?: (childKey: string) => void
}
```

### QuickContext 接口

```typescript
interface QuickContext {
  /** 触发外部自定义动作 */
  emitAction: (key: string, childKey?: string) => void
}
```

### useGlobalModal API

```typescript
const { open, close, confirm, info, success, warning, error } = useGlobalModal()

// 打开自定义组件弹窗
open({
  title: '标题',
  component: MyComponent,
  componentProps: { prop1: 'value' },
  preset: 'card',
  width: 480,
})

// 确认弹窗
confirm({
  title: '确认删除？',
  content: '此操作不可撤销',
  onPositiveClick: () => {
    /* 确认逻辑 */
  },
})

// 信息提示
info({ title: '提示', content: '操作成功' })
success({ title: '成功', content: '保存完成' })
warning({ title: '警告', content: '请注意' })
error({ title: '错误', content: '操作失败' })
```

## 完整示例：工具箱菜单

### 配置

```typescript
// src/config/ui.ts
{
  key: 'toolbox',
  display: 'dropdown',
  type: 'menu',
  icon: 'Wrench',
  label: '工具箱',
  children: [
    { key: 'calculator', label: '计算器', icon: 'Calculator' },
    { key: 'calendar', label: '日历', icon: 'Calendar' },
    { key: 'notebook', label: '记事本', icon: 'Notebook' },
  ],
}
```

### 组合式函数

```typescript
// src/composables/quick/useQuickToolbox.ts
import { computed } from 'vue'
import { useGlobalModal } from '@/composables/useGlobalModal'
import CalculatorModal from '@/components/quick/CalculatorModal.vue'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

export function useQuickToolbox(config: QuickConfigItem, context: QuickContext): UseQuickReturn {
  const { open } = useGlobalModal()

  const isActive = computed(() => false)

  function onClick(): void {
    // action 类型时触发
  }

  function onSelect(childKey: string): void {
    switch (childKey) {
      case 'calculator':
        open({
          title: '计算器',
          component: CalculatorModal,
          preset: 'card',
          width: 360,
        })
        break
      case 'calendar':
        // 跳转到日历页面
        context.emitAction(config.key, childKey)
        break
      case 'notebook':
        context.emitAction(config.key, childKey)
        break
    }
  }

  return {
    isActive,
    currentIcon: computed(() => config.icon),
    onClick,
    onSelect,
  }
}
```

## 高级用法

### 动态修改 Quick 配置

```typescript
import { useUIStore } from '@/store'

const uiStore = useUIStore()

// 更新 QuickBar 配置
uiStore.updateQuickBarItems([
  ...uiStore.quickBarConfig,
  { key: 'new', display: 'visible', type: 'action', icon: 'Plus', label: '新建' },
])
```

### 条件显示按钮

```typescript
// 在组件中根据权限动态显示
import { computed } from 'vue'
import { useAuthStore } from '@/store'

const authStore = useAuthStore()

const quickBarItems = computed(() => {
  const items = [{ key: 'home', display: 'visible', type: 'action', icon: 'Home', label: '首页' }]

  // 仅管理员显示管理按钮
  if (authStore.isAdmin) {
    items.push({
      key: 'admin',
      display: 'dropdown',
      type: 'menu',
      icon: 'Shield',
      label: '管理',
      children: [
        { key: 'users', label: '用户管理', icon: 'Users' },
        { key: 'settings', label: '系统设置', icon: 'Settings' },
      ],
    })
  }

  return items
})
```

## 与内置 Quick 的关系

### 内置 Quick 的处理

内置 Quick（sidebar、footer、theme、user）的处理逻辑已经封装在对应的组合式函数中：

- `useQuickLayout` - 处理 sidebar 和 footer
- `useQuickTheme` - 处理 theme
- `useQuickUser` - 处理 user

这些内置 Quick 会自动响应点击事件，无需额外配置。

### 自定义 Quick 与内置 Quick 的区别

| 特性     | 内置 Quick           | 自定义 Quick                 |
| -------- | -------------------- | ---------------------------- |
| 配置位置 | `src/config/ui.ts`   | `src/config/ui.ts`           |
| 处理逻辑 | 已封装在组合式函数中 | 需要创建组合式函数           |
| 弹窗集成 | 内部处理             | 通过 `useGlobalModal` 处理   |
| 开发步骤 | 仅配置               | 配置 + 组合式函数 + 弹窗组件 |

## 调试技巧

### 查看当前配置

```typescript
import { useUIStore } from '@/store'

const uiStore = useUIStore()
console.log('QuickBar配置:', uiStore.quickBarConfig)
```

### 检查工厂注册

```typescript
// 在 QuickBar.vue 中打印
console.log('已注册的Quick工厂:', Object.keys(quickFactories))
```

## 常见问题

### Q: 点击按钮没有反应？

A: 检查以下几点：

1. **是否注册了组合式函数**：

```typescript
// QuickBar.vue
const quickFactories = {
  // ...
  notification: useQuickNotification, // 是否已注册
}
```

2. **key 是否匹配**：

```typescript
// ui.ts 中的 key 必须与工厂注册的 key 一致
{ key: 'notification', ... }  // ✅
{ key: 'notify', ... }        // ❌ 不匹配
```

3. **组合式函数是否正确导出**：

```typescript
// useQuickNotification.ts
export function useQuickNotification(...) {  // ✅ 命名导出
```

### Q: 弹窗没有显示？

A: 检查以下几点：

1. **MainLayout.vue 中是否有 GlobalModal**：

```vue
<!-- MainLayout.vue -->
<GlobalModal v-model:show="modalState.visible" ... />
```

2. **useGlobalModal 调用是否正确**：

```typescript
const { open } = useGlobalModal()
open({ ... })  // ✅ 调用 open
```

### Q: 按钮图标不显示？

A: 检查图标名称是否正确：

```typescript
// ✅ 正确
{
  icon: 'Settings'
}

// ❌ 错误
{
  icon: 'settings'
} // 大小写敏感
{
  icon: 'cog'
} // 不存在的图标
```

## 下一步

- [接入真实后端](./backend-integration.md) - 连接真实 API
