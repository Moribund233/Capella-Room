# 框架介绍

## 设计理念

CapellaUI 采用**配置驱动**的设计理念，将UI的呈现与业务逻辑分离。开发者只需通过简单的配置对象，即可定义整个应用的界面结构，而无需深入修改组件代码。

### 核心思想

```
配置（Config） → 状态（Store） → 呈现（Component）
```

1. **配置层** - 定义UI的结构和初始状态
2. **状态层** - 管理UI的运行时状态和用户偏好
3. **呈现层** - 根据状态渲染界面，处理用户交互

## 架构特点

### 1. 配置驱动

通过配置文件定义UI元素，而非硬编码在组件中：

```typescript
// src/config/ui.ts
export const sidebarConfig = {
  items: [
    { name: '首页', icon: 'LayoutDashboard', path: '/home' },
    { name: '数据', icon: 'Database', path: '/data' },
  ],
}
```

### 2. 分层架构

```
┌─────────────────────────────────────┐
│           呈现层 (Views)             │
│    ExampleView.vue / HomeView.vue   │
├─────────────────────────────────────┤
│           组件层 (Components)        │
│  QuickBar / DockBar / AppSideBar    │
├─────────────────────────────────────┤
│           状态层 (Store)             │
│    useUIStore / useAuthStore        │
├─────────────────────────────────────┤
│           配置层 (Config)            │
│    ui.ts / useConfig()              │
├─────────────────────────────────────┤
│           API层                      │
│    uiApi / authApi / request        │
└─────────────────────────────────────┘
```

### 3. 事件驱动通信

使用 Mitt 事件总线实现组件间解耦通信：

```typescript
// QuickBar 触发事件
quickActionBus.emit('quick-action', { key: 'user', childKey: 'profile' })

// 任意组件监听事件
quickActionBus.on('quick-action', (event) => {
  if (event.key === 'user') {
    // 处理用户操作
  }
})
```

## 核心组件

### 布局组件

| 组件       | 说明       | 位置                             |
| ---------- | ---------- | -------------------------------- |
| MainLayout | 主布局容器 | components/layout/MainLayout.vue |
| AppHeader  | 顶部导航栏 | components/layout/AppHeader.vue  |
| AppSideBar | 侧边栏     | components/layout/AppSideBar.vue |
| AppFooter  | 底部栏     | components/layout/AppFooter.vue  |

### 功能组件

| 组件     | 说明           | 位置                           |
| -------- | -------------- | ------------------------------ |
| QuickBar | 顶部快捷按钮栏 | components/common/QuickBar.vue |
| DockBar  | 页面级浮动导航 | components/common/DockBar.vue  |

## 配置系统

### 配置优先级

配置采用**分层覆盖**策略：

```
运行时本地配置 > 云端配置 > 默认配置
```

1. **默认配置** - `src/config/ui.ts` 中定义的静态配置
2. **云端配置** - 从后端API获取的用户配置（预留）
3. **本地配置** - 存储在 localStorage 的用户偏好设置

### 配置类型

```typescript
// UI 配置结构
interface UIConfig {
  app: AppConfig // 应用信息
  sidebar: SidebarConfig // 侧边栏菜单
  theme: ThemeConfig // 主题设置
  dock: DockConfig // Dock栏配置
  quickBar: QuickItemConfig[] // 快捷按钮
}
```

## 响应式设计

框架内置三端响应式适配：

| 断点   | 宽度范围       | 特点                 |
| ------ | -------------- | -------------------- |
| 移动端 | < 768px        | 侧边栏隐藏，汉堡菜单 |
| 平板端 | 768px - 1024px | 可折叠侧边栏         |
| 桌面端 | > 1024px       | 固定侧边栏           |

## 主题系统

支持浅色/深色主题切换：

```typescript
// 切换主题
const { setTheme } = useTheme()
setTheme('dark') // 或 'light'
```

主题配置通过 CSS 变量实现，可在 `src/styles/variables.css` 中自定义。

## 扩展性设计

### 添加新的 Quick 按钮

1. 在配置中定义
2. 在组件中监听事件
3. 无需修改框架代码

### 添加新的页面

1. 创建页面组件
2. 配置路由
3. 配置侧边栏菜单
4. 可选：配置 Dock 栏

## 与类似框架对比

| 特性       | CapellaUI    | Ant Design Pro | Vue-Vben-Admin |
| ---------- | ------------- | -------------- | -------------- |
| 配置驱动   | ✅ 核心设计   | ⚠️ 部分支持    | ⚠️ 部分支持    |
| 类型安全   | ✅ TypeScript | ✅ TypeScript  | ✅ TypeScript  |
| 响应式     | ✅ 内置       | ✅ 内置        | ✅ 内置        |
| 学习曲线   | 🟢 平缓       | 🟡 中等        | 🟡 中等        |
| 自定义难度 | 🟢 简单       | 🟡 中等        | 🟡 中等        |

## 适用场景

- 管理后台系统
- 数据展示平台
- 配置化管理界面
- 需要快速原型开发的项目

## 不适用场景

- 高度定制化的C端页面
- 对首屏加载速度要求极高的应用
- 需要复杂动画效果的项目
