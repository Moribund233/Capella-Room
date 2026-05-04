# 配置系统

## 概述

SeredeliUI 采用**配置驱动**的设计理念，通过集中式的配置文件定义整个应用的 UI 结构。这种设计让开发者无需深入组件代码，即可快速调整界面布局和样式。

## 配置文件

所有 UI 配置集中在 `src/config/ui.ts` 文件中：

```typescript
// src/config/ui.ts
export const uiConfig = {
  app: appConfig,        // 应用信息
  sidebar: sidebarConfig, // 侧边栏
  theme: themeConfig,    // 主题
  quickBar: quickBarConfig, // 快捷按钮
  dock: dockConfig,      // Dock 栏
}
```

## 配置类型详解

### 1. 应用配置（AppConfig）

定义应用的基本信息：

```typescript
export const appConfig: AppConfig = {
  name: 'SeredeliUI',      // 应用名称
  logo: '/favicon.svg',    // Logo 路径
  version: '1.0.0',        // 版本号
}
```

**效果**：显示在页面标题栏和浏览器标签页。

### 2. 侧边栏配置（SidebarConfig）

定义侧边栏菜单项：

```typescript
export const sidebarConfig: { items: SidebarItemConfig[] } = {
  items: [
    {
      name: '首页',              // 显示文本
      icon: 'LayoutDashboard',   // Lucide 图标名
      path: '/home',             // 路由路径
    },
    {
      name: '数据管理',
      icon: 'Database',
      path: '/data',
    },
  ],
}
```

**图标选择**：从 [Lucide Icons](https://lucide.dev/icons/) 选择图标名称。

### 3. 主题配置（ThemeConfig）

定义默认主题：

```typescript
export const themeConfig: ThemeConfig = {
  name: 'dark',  // 'light' 或 'dark'
}
```

**动态切换**：用户可以通过 QuickBar 的主题按钮切换。

### 4. QuickBar 配置（QuickItemConfig[]）

定义顶部快捷按钮：

```typescript
export const quickBarConfig: QuickItemConfig[] = [
  // Action 类型：直接执行动作
  {
    key: 'sidebar',           // 唯一标识
    display: 'visible',       // 'visible' 或 'dropdown'
    type: 'action',           // 'action' 或 'menu'
    icon: 'PanelLeft',        // 主图标
    iconAlt: 'PanelRight',    // 替代图标（状态切换时使用）
    label: '切换侧边栏',       // 提示文本
  },
  // Menu 类型：带下拉菜单
  {
    key: 'user',
    display: 'dropdown',
    type: 'menu',
    icon: 'UserCircle',
    iconAlt: 'User',
    label: '用户中心',
    children: [               // 子菜单项
      { key: 'profile', label: '用户详情', icon: 'User' },
      { key: 'logout', label: '登出', icon: 'LogOut' },
    ],
  },
]
```

**配置说明**：

| 属性 | 类型 | 说明 |
|------|------|------|
| key | string | 唯一标识，用于事件处理 |
| display | 'visible' \| 'dropdown' | 显示方式：外显或聚合到菜单 |
| type | 'action' \| 'menu' | 按钮类型：直接执行或有子菜单 |
| icon | string | 主图标（Lucide 图标名） |
| iconAlt | string | 替代图标（可选，用于状态切换） |
| label | string | 显示文本 |
| children | array | 子菜单项（仅 menu 类型有效） |

### 5. Dock 配置（DockConfig）

定义页面级浮动导航：

```typescript
export const dockConfig: DockConfig = {
  // 页面 key（对应路由第一级路径）
  example: {
    enabled: true,           // 是否启用
    position: 'bottom',      // 位置：'bottom' | 'left' | 'right'
    offset: 24,              // 距离边缘距离（像素）
    items: [                 // Dock 项列表
      {
        key: 'overview',     // 唯一标识
        label: '概览',       // 显示文本
        icon: 'Layout',      // Lucide 图标名
        path: '/example/overview',  // 路由路径
      },
      {
        key: 'icons',
        label: '图标',
        icon: 'Smile',
        path: '/example/icons',
      },
    ],
  },
}
```

**自动匹配**：Dock 栏会根据当前路由自动匹配对应的配置。

## 配置优先级

配置采用**分层覆盖**策略：

```
运行时本地配置 > 云端配置 > 默认配置
```

### 默认配置

来自 `src/config/ui.ts` 的静态配置。

### 云端配置（预留）

从后端 API 获取的用户配置，用于多设备同步。

### 本地配置

存储在 localStorage 的用户偏好设置，如：

- 侧边栏折叠状态
- 主题偏好
- 自定义快捷按钮顺序

## 在组件中读取配置

### 使用组合式函数

```typescript
import { useConfig, useAppConfig, useSidebarConfig } from '@/composables'

// 获取完整配置
const { config } = useConfig()
console.log(config.value.sidebar.items)

// 获取应用配置
const appConfig = useAppConfig()
console.log(appConfig.name)

// 获取侧边栏配置
const { menuItems } = useSidebarConfig()
```

### 使用 Store

```typescript
import { useUIStore } from '@/store'

const uiStore = useUIStore()

// 读取合并后的配置
console.log(uiStore.mergedConfig)

// 读取特定配置
console.log(uiStore.sidebarConfig)
console.log(uiStore.quickBarConfig)
console.log(uiStore.dockConfig)

// 更新配置
uiStore.updateSidebarItems(newItems)
uiStore.updateThemeConfig({ name: 'dark' })
```

## 配置类型定义

```typescript
// 侧边栏菜单项
interface SidebarItemConfig {
  name: string      // 显示名称
  icon: string      // Lucide 图标名
  path: string      // 路由路径
}

// QuickBar 项
interface QuickItemConfig {
  key: string
  display: 'visible' | 'dropdown'
  type: 'action' | 'menu'
  icon: string
  iconAlt?: string
  label: string
  children?: QuickChildItemConfig[]
}

// Dock 页面配置
interface DockPageConfig {
  enabled?: boolean
  position?: 'bottom' | 'left' | 'right'
  offset?: number
  items?: DockItemConfig[]
}

// Dock 项
interface DockItemConfig {
  key: string
  label: string
  icon: string
  path: string
  disabled?: boolean
}
```

## 最佳实践

### 1. 保持配置简洁

不要在配置中添加过多逻辑，配置应该是声明式的：

```typescript
// ✅ 好的做法
{
  name: '数据管理',
  icon: 'Database',
  path: '/data',
}

// ❌ 避免
{
  name: userStore.isAdmin ? '数据管理' : '我的数据',
  icon: getIconForRole(),
  path: computedPath(),
}
```

### 2. 使用语义化的 key

```typescript
// ✅ 好的做法
{ key: 'user-profile', label: '用户详情' }
{ key: 'data-export', label: '数据导出' }

// ❌ 避免
{ key: 'item1', label: '用户详情' }
{ key: 'btn2', label: '数据导出' }
```

### 3. 图标命名规范

使用 Lucide 图标的标准命名：

```typescript
// ✅ 正确
{ icon: 'Settings' }
{ icon: 'UserCircle' }

// ❌ 错误
{ icon: 'settings' }      // 小写
{ icon: 'user-circle' }   //  kebab-case
```

## 下一步

- [QuickBar 开发](./quickbar-development.md) - 自定义快捷按钮
- [接入真实后端](./backend-integration.md) - 连接真实 API
