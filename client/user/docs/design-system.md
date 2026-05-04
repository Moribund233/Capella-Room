# UI 设计规范

## 设计理念

Seredeli Room 采用**现代简约**与**流畅动效**相结合的设计语言，参考微信的熟悉交互模式，同时融入更精致的视觉层次和动画效果。设计核心原则：

- **清晰**：信息层级分明，用户一眼找到重点
- **流畅**：动画过渡自然，操作反馈即时
- **一致**：跨端体验统一，交互模式可预期
- **精致**：细节打磨到位，微交互丰富

## 色彩系统

### 主色调

| 名称 | 色值 | 用途 |
|------|------|------|
| Primary | `#07C160` | 主按钮、发送按钮、在线状态 |
| Primary Hover | `#06AD56` | 主按钮悬停 |
| Primary Pressed | `#05944F` | 主按钮按下 |
| Primary Light | `#E6F7ED` | 浅色背景、选中状态 |

### 功能色

| 名称 | 色值 | 用途 |
|------|------|------|
| Success | `#07C160` | 成功提示、已发送状态 |
| Warning | `#FAAD14` | 警告提示、待处理状态 |
| Error | `#FF4D4F` | 错误提示、删除操作 |
| Info | `#1890FF` | 信息提示、链接 |

### 中性色

| 名称 | 色值 | 用途 |
|------|------|------|
| Text Primary | `#262626` | 主要文字 |
| Text Secondary | `#595959` | 次要文字 |
| Text Tertiary | `#8C8C8C` | 辅助文字、时间戳 |
| Text Quaternary | `#BFBFBF` | 占位符、禁用状态 |
| Border | `#D9D9D9` | 边框、分割线 |
| Divider | `#F0F0F0` | 细分割线 |
| Background | `#F5F5F5` | 页面背景 |
| Background Light | `#FAFAFA` | 卡片背景 |
| White | `#FFFFFF` | 纯白背景 |

### 深色模式（预留）

```typescript
// 深色模式配色（后续实现）
const darkTheme = {
  background: '#141414',
  surface: '#1F1F1F',
  textPrimary: '#FFFFFF',
  textSecondary: '#B3B3B3',
  border: '#434343',
}
```

## 字体系统

### 字体栈

```css
/* 中文优先 */
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Hiragino Sans GB', 
             'Microsoft YaHei', 'Helvetica Neue', Helvetica, Arial, sans-serif;

/* 等宽字体（代码、时间） */
font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
```

### 字号规范

| 级别 | 字号 | 行高 | 字重 | 用途 |
|------|------|------|------|------|
| H1 | 24px | 32px | 600 | 页面标题 |
| H2 | 20px | 28px | 600 | 区块标题 |
| H3 | 16px | 24px | 600 | 卡片标题 |
| Body | 14px | 22px | 400 | 正文内容 |
| Small | 12px | 20px | 400 | 辅助文字、时间戳 |
| Tiny | 10px | 16px | 400 | 徽章、标签 |

## 间距系统

### 基础间距

| Token | 值 | 用途 |
|-------|-----|------|
| space-xs | 4px | 图标与文字间距 |
| space-sm | 8px | 紧凑元素间距 |
| space-md | 12px | 标准元素间距 |
| space-lg | 16px | 卡片内边距 |
| space-xl | 20px | 区块间距 |
| space-2xl | 24px | 大区块间距 |
| space-3xl | 32px | 页面边距 |

### 圆角规范

| Token | 值 | 用途 |
|-------|-----|------|
| radius-sm | 4px | 小按钮、标签 |
| radius-md | 8px | 按钮、输入框 |
| radius-lg | 12px | 卡片、弹窗 |
| radius-xl | 16px | 大卡片、模态框 |
| radius-full | 9999px | 头像、胶囊按钮 |

## 阴影系统

| 级别 | 值 | 用途 |
|------|-----|------|
| Shadow SM | `0 1px 2px rgba(0,0,0,0.05)` | 轻微提升 |
| Shadow MD | `0 4px 6px -1px rgba(0,0,0,0.1)` | 卡片、下拉菜单 |
| Shadow LG | `0 10px 15px -3px rgba(0,0,0,0.1)` | 弹窗、模态框 |
| Shadow XL | `0 20px 25px -5px rgba(0,0,0,0.1)` | 悬浮按钮、抽屉 |

## 动画系统

### 缓动函数

| 名称 | 值 | 用途 |
|------|-----|------|
| ease-default | `cubic-bezier(0.4, 0, 0.2, 1)` | 默认过渡 |
| ease-in | `cubic-bezier(0.4, 0, 1, 1)` | 进入动画 |
| ease-out | `cubic-bezier(0, 0, 0.2, 1)` | 退出动画 |
| ease-bounce | `cubic-bezier(0.68, -0.55, 0.265, 1.55)` | 弹性效果 |
| ease-spring | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` | 弹簧效果 |

### 动画时长

| Token | 值 | 用途 |
|-------|-----|------|
| duration-fast | 150ms | 微交互（按钮反馈） |
| duration-normal | 250ms | 标准过渡 |
| duration-slow | 350ms | 复杂动画 |
| duration-slower | 500ms | 页面切换 |

### 关键动画效果

#### 1. 消息气泡动画

```css
/* 发送消息 - 从右滑入 */
@keyframes messageSendIn {
  from {
    opacity: 0;
    transform: translateX(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateX(0) scale(1);
  }
}

/* 接收消息 - 从左滑入 */
@keyframes messageReceiveIn {
  from {
    opacity: 0;
    transform: translateX(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateX(0) scale(1);
  }
}
```

#### 2. 页面切换动画

```css
/* 移动端页面滑动 */
@keyframes pageSlideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

@keyframes pageSlideOut {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(-30%);
  }
}
```

#### 3. 列表加载动画

```css
/* 骨架屏闪烁 */
@keyframes skeletonShimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

/* 列表项渐入 */
@keyframes listItemFadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

#### 4. 按钮反馈动画

```css
/* 按钮点击波纹 */
@keyframes ripple {
  from {
    transform: scale(0);
    opacity: 0.5;
  }
  to {
    transform: scale(4);
    opacity: 0;
  }
}

/* 发送按钮成功状态 */
@keyframes sendSuccess {
  0% {
    transform: scale(1);
  }
  50% {
    transform: scale(0.9);
  }
  100% {
    transform: scale(1);
  }
}
```

## 组件规范

### 按钮

#### 主按钮

```typescript
// 样式属性
{
  backgroundColor: '#07C160',
  color: '#FFFFFF',
  borderRadius: '8px',
  padding: '8px 16px',
  fontSize: '14px',
  fontWeight: 500,
  transition: 'all 250ms cubic-bezier(0.4, 0, 0.2, 1)',
  
  // 状态
  hover: { backgroundColor: '#06AD56' },
  active: { backgroundColor: '#05944F', transform: 'scale(0.98)' },
  disabled: { backgroundColor: '#D9D9D9', color: '#BFBFBF' }
}
```

#### 次按钮

```typescript
{
  backgroundColor: '#FFFFFF',
  color: '#262626',
  border: '1px solid #D9D9D9',
  borderRadius: '8px',
  padding: '8px 16px',
  
  hover: { borderColor: '#07C160', color: '#07C160' },
  active: { backgroundColor: '#E6F7ED' }
}
```

#### 文字按钮

```typescript
{
  backgroundColor: 'transparent',
  color: '#07C160',
  padding: '4px 8px',
  
  hover: { backgroundColor: '#E6F7ED' }
}
```

### 输入框

```typescript
// 聊天输入框
{
  backgroundColor: '#F5F5F5',
  border: 'none',
  borderRadius: '8px',
  padding: '10px 14px',
  fontSize: '14px',
  minHeight: '40px',
  maxHeight: '120px',
  
  focus: { backgroundColor: '#FFFFFF', boxShadow: '0 0 0 2px rgba(7,193,96,0.2)' }
}
```

### 消息气泡

#### 发送的消息

```typescript
{
  backgroundColor: '#95EC69',  // 微信绿
  color: '#262626',
  borderRadius: '8px 2px 8px 8px',
  padding: '10px 14px',
  maxWidth: '70%',
  fontSize: '14px',
  lineHeight: '22px',
  boxShadow: '0 1px 2px rgba(0,0,0,0.05)'
}
```

#### 接收的消息

```typescript
{
  backgroundColor: '#FFFFFF',
  color: '#262626',
  borderRadius: '2px 8px 8px 8px',
  padding: '10px 14px',
  maxWidth: '70%',
  fontSize: '14px',
  lineHeight: '22px',
  boxShadow: '0 1px 2px rgba(0,0,0,0.05)'
}
```

### 头像

```typescript
// 尺寸规范
{
  xs: { size: 24, radius: 'full' },   // 列表内小头像
  sm: { size: 32, radius: 'full' },   // 消息头像
  md: { size: 40, radius: 'full' },   // 聊天室头像
  lg: { size: 56, radius: 'full' },   // 个人中心
  xl: { size: 80, radius: 'full' }    // 大头像展示
}
```

### 徽章

```typescript
// 未读消息徽章
{
  backgroundColor: '#FF4D4F',
  color: '#FFFFFF',
  borderRadius: '9999px',
  minWidth: '18px',
  height: '18px',
  padding: '0 6px',
  fontSize: '12px',
  fontWeight: 500,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center'
}
```

## 布局规范

### 桌面端布局 (>1024px)

```
┌─────────────────────────────────────────────────────────────┐
│  Sidebar (280px)    │        Chat Area (flex: 1)           │
│  ┌───────────────┐  │  ┌─────────────────────────────────┐ │
│  │    Header     │  │  │           Header                │ │
│  ├───────────────┤  │  ├─────────────────────────────────┤ │
│  │               │  │  │                                 │ │
│  │   Room List   │  │  │      Message List               │ │
│  │               │  │  │                                 │ │
│  │   (scroll)    │  │  │      (virtual scroll)           │ │
│  │               │  │  │                                 │ │
│  ├───────────────┤  │  ├─────────────────────────────────┤ │
│  │    Footer     │  │  │         Input Area              │ │
│  └───────────────┘  │  └─────────────────────────────────┘ │
└─────────────────────┴───────────────────────────────────────┘
```

### 平板端布局 (768px-1024px)

```
┌─────────────────────────────────────────────────────────────┐
│  Collapsible Sidebar (64px collapsed / 240px expanded)      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ │ Room List (overlay when expanded)                   │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │                    Chat Area                          │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 手机端布局 (<768px)

```
┌─────────────────────────────────────────────────────────────┐
│  Room List View (默认)                                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Header (搜索栏)                                       │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ [头像] 聊天室名称                    [时间]    │  │  │
│  │  │        最后消息预览...                 [徽章]   │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ [头像] 另一个聊天室                             │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Chat View (点击进入)                                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  [←] 聊天室名称                    [更多 ⋮]           │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │                    Messages                           │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  [+] [输入框...                    ] [发送]           │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 图标规范

### 图标库

使用 **Lucide Vue** 图标库，保持图标风格统一。

```typescript
// 安装
// pnpm add lucide-vue-next

// 使用
import { MessageSquare, Send, Settings, User } from 'lucide-vue-next'
```

### 图标尺寸

| 尺寸 | 用途 |
|------|------|
| 16px | 按钮内图标、列表图标 |
| 20px | 导航图标、工具栏图标 |
| 24px | 大按钮、空状态图标 |
| 32px | 功能入口、特色图标 |

### 常用图标映射

| 功能 | 图标名称 |
|------|----------|
| 消息 | `MessageSquare` |
| 发送 | `Send` |
| 用户 | `User` |
| 设置 | `Settings` |
| 搜索 | `Search` |
| 添加 | `Plus` |
| 删除 | `Trash2` |
| 编辑 | `Pencil` |
| 返回 | `ChevronLeft` |
| 更多 | `MoreVertical` |
| 表情 | `Smile` |
| 文件 | `Paperclip` |
| 图片 | `Image` |
| 在线 | `Circle` (fill) |
| 离线 | `Circle` |
| 通知 | `Bell` |
| 退出 | `LogOut` |

## 响应式适配规则

### 断点检测

```typescript
// useResponsive composable
const breakpoints = {
  isMobile: width < 768,
  isTablet: width >= 768 && width < 1024,
  isDesktop: width >= 1024,
  isLargeDesktop: width >= 1280
}
```

### 适配策略

| 元素 | 桌面端 | 平板端 | 手机端 |
|------|--------|--------|--------|
| 侧边栏 | 固定 280px | 可折叠 64px/240px | 隐藏，抽屉式 |
| 消息气泡最大宽度 | 60% | 70% | 80% |
| 头像尺寸 | 40px | 36px | 32px |
| 输入框高度 | 自适应 | 自适应 | 固定最小高度 |
| 页面切换 | 无动画 | 无动画 | 滑动动画 |
| 右键菜单 | 支持 | 支持 | 长按菜单 |

## 可访问性规范

### 颜色对比度

- 文字与背景对比度至少 4.5:1
- 大文字（18px+）对比度至少 3:1
- 交互元素对比度至少 3:1

### 焦点状态

```css
/* 焦点环 */
:focus-visible {
  outline: 2px solid #07C160;
  outline-offset: 2px;
}
```

### 动画偏好

```css
/* 尊重用户的减少动画偏好 */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

## Naive UI 主题配置

```typescript
// src/theme/index.ts
import type { GlobalThemeOverrides } from 'naive-ui'

export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#07C160',
    primaryColorHover: '#06AD56',
    primaryColorPressed: '#05944F',
    primaryColorSuppl: '#07C160',
    
    textColorBase: '#262626',
    textColor1: '#262626',
    textColor2: '#595959',
    textColor3: '#8C8C8C',
    
    borderColor: '#D9D9D9',
    dividerColor: '#F0F0F0',
    
    fontSize: '14px',
    fontSizeMini: '12px',
    fontSizeTiny: '12px',
    fontSizeSmall: '14px',
    fontSizeMedium: '14px',
    fontSizeLarge: '16px',
    fontSizeHuge: '20px',
  },
  Button: {
    borderRadiusSmall: '4px',
    borderRadiusMedium: '8px',
    borderRadiusLarge: '8px',
  },
  Input: {
    borderRadius: '8px',
    heightMedium: '40px',
  },
  Card: {
    borderRadius: '12px',
  },
  Modal: {
    borderRadius: '16px',
  }
}
```
