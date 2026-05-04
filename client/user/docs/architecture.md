# 技术架构设计

## 架构概览

本项目采用 Vue 3 + TypeScript 的现代前端架构，遵循组合式 API 风格，结合 Pinia 状态管理和 Vue Router 路由管理，构建一个可扩展、易维护的实时聊天客户端。

```
┌─────────────────────────────────────────────────────────────┐
│                        应用层 (Pages)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Login     │  │   Chat      │  │      Profile        │  │
│  │    Page     │  │    Page     │  │       Page          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      业务组件层 (Components)                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ ChatSidebar │  │ ChatWindow  │  │   MessageBubble     │  │
│  │  RoomList   │  │  InputBox   │  │   UserAvatar        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      组合式函数层 (Composables)               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  useAuth    │  │  useWebSocket│  │    useMessage      │  │
│  │  useRoom    │  │  useUser    │  │    useNotification │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                       状态管理层 (Pinia Stores)               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  authStore  │  │  roomStore  │  │    messageStore     │  │
│  │  userStore  │  │  uiStore    │  │    websocketStore   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                        服务层 (Services)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  httpClient │  │ wsService   │  │   storageService    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                        工具层 (Utils)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   format    │  │  validate   │  │      date           │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 核心架构决策

### 1. 状态管理策略

#### Pinia Store 设计原则

| Store 名称 | 职责 | 持久化 |
|------------|------|--------|
| `authStore` | 认证状态、Token 管理 | 是 (localStorage) |
| `userStore` | 当前用户信息、用户列表 | 否 |
| `roomStore` | 聊天室列表、当前房间 | 否 |
| `messageStore` | 消息缓存、未读计数 | 部分 (sessionStorage) |
| `websocketStore` | WebSocket 连接状态 | 否 |
| `uiStore` | UI 状态（主题、侧边栏折叠等） | 是 (localStorage) |

#### 状态流转图

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   用户操作    │────▶│   Action     │────▶│   Store      │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                 │
                                                 ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   组件更新    │◀────│   响应式订阅  │◀────│   State      │
└──────────────┘     └──────────────┘     └──────────────┘
```

### 2. WebSocket 架构

#### 连接管理

```typescript
// WebSocket 连接状态机
enum WebSocketState {
  CONNECTING = 'CONNECTING',    // 正在连接
  CONNECTED = 'CONNECTED',      // 已连接
  RECONNECTING = 'RECONNECTING', // 正在重连
  DISCONNECTED = 'DISCONNECTED', // 已断开
}
```

#### 消息处理流程

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Server    │───▶│   WS Service│───▶│  Message    │───▶│   Store     │
│             │    │   (接收)    │    │  Handler    │    │  (更新状态) │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                              │
                                                              ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Server    │◀───│   WS Service│◀───│   Action    │◀───│   Component │
│             │    │   (发送)    │    │   Dispatch  │    │  (用户操作) │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

#### 心跳机制

- 心跳间隔：30 秒
- 超时时间：10 秒
- 重连策略：指数退避（1s, 2s, 4s, 8s... 最大 30s）

### 3. HTTP API 架构

#### 请求拦截器

```typescript
// 请求拦截器职责
1. 自动附加 Access Token
2. 请求日志记录（开发环境）
3. 请求防抖/节流处理
```

#### 响应拦截器

```typescript
// 响应拦截器职责
1. 统一错误处理
2. Token 过期自动刷新
3. 响应数据标准化
4. 错误提示统一封装
```

#### 错误处理策略

| 错误码 | 处理方式 |
|--------|----------|
| 401 Unauthorized | 尝试刷新 Token，失败则跳转登录 |
| 403 Forbidden | 提示权限不足 |
| 404 Not Found | 提示资源不存在 |
| 409 Conflict | 提示资源冲突（如用户名已存在） |
| 422 Validation | 表单验证错误，显示具体字段错误 |
| 500+ Server | 提示服务器错误，记录日志 |

### 4. 路由架构

#### 路由结构

```typescript
// 路由配置结构
const routes = [
  {
    path: '/login',
    component: LoginView,
    meta: { public: true }
  },
  {
    path: '/register',
    component: RegisterView,
    meta: { public: true }
  },
  {
    path: '/',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        component: ChatView,
        children: [
          { path: 'room/:roomId', component: ChatRoomView }
        ]
      },
      { path: 'profile', component: ProfileView },
      { path: 'settings', component: SettingsView }
    ]
  }
]
```

#### 路由守卫

```typescript
// 全局前置守卫职责
1. 检查认证状态
2. 处理 Token 过期
3. 权限验证
4. 路由过渡动画准备
```

### 5. 响应式架构

#### 断点定义

```typescript
// 响应式断点（与 Naive UI 保持一致）
const breakpoints = {
  xs: 0,      // 手机竖屏
  s: 640,     // 手机横屏
  m: 768,     // 平板竖屏
  l: 1024,    // 平板横屏/小桌面
  xl: 1280,   // 桌面
  xxl: 1536,  // 大桌面
}
```

#### 布局适配策略

| 断点 | 布局模式 | 侧边栏 | 聊天区域 |
|------|----------|--------|----------|
| < 768px (xs, s) | 移动端 | 全屏抽屉 | 全屏 |
| 768px - 1024px (m, l) | 平板端 | 可折叠侧边栏 | 自适应 |
| > 1024px (xl, xxl) | 桌面端 | 固定侧边栏 | 自适应 |

## 核心 Composables 设计

### 1. useAuth - 认证管理

```typescript
// 职责：处理登录、注册、Token 刷新、登出
interface UseAuthReturn {
  isAuthenticated: ComputedRef<boolean>
  user: ComputedRef<User | null>
  login: (credentials: LoginCredentials) => Promise<void>
  register: (data: RegisterData) => Promise<void>
  logout: () => Promise<void>
  refreshToken: () => Promise<boolean>
}
```

### 2. useWebSocket - WebSocket 连接

```typescript
// 职责：管理 WebSocket 连接、消息收发
interface UseWebSocketReturn {
  state: Ref<WebSocketState>
  connect: () => void
  disconnect: () => void
  send: (message: WebSocketMessage) => boolean
  onMessage: (handler: MessageHandler) => void
  onError: (handler: ErrorHandler) => void
}
```

### 3. useRoom - 聊天室管理

```typescript
// 职责：聊天室列表、加入/离开房间
interface UseRoomReturn {
  rooms: ComputedRef<Room[]>
  currentRoom: ComputedRef<Room | null>
  joinRoom: (roomId: string) => Promise<void>
  leaveRoom: (roomId: string) => Promise<void>
  createRoom: (data: CreateRoomData) => Promise<Room>
}
```

### 4. useMessage - 消息管理

```typescript
// 职责：消息发送、接收、历史加载
interface UseMessageReturn {
  messages: ComputedRef<Message[]>
  sendMessage: (content: string, replyTo?: string) => void
  loadHistory: (before?: string) => Promise<boolean>
  editMessage: (messageId: string, content: string) => void
  deleteMessage: (messageId: string) => void
}
```

### 5. useResponsive - 响应式适配

```typescript
// 职责：响应式断点检测、布局模式
interface UseResponsiveReturn {
  breakpoint: ComputedRef<Breakpoint>
  isMobile: ComputedRef<boolean>
  isTablet: ComputedRef<boolean>
  isDesktop: ComputedRef<boolean>
  sidebarCollapsed: Ref<boolean>
}
```

## 性能优化策略

### 1. 代码分割

```typescript
// 路由级别代码分割
const ChatView = () => import('@/views/ChatView.vue')
const ProfileView = () => import('@/views/ProfileView.vue')

// 组件级别代码分割
const EmojiPicker = () => import('@/components/EmojiPicker.vue')
```

### 2. 虚拟滚动

```typescript
// 消息列表虚拟滚动
// 使用 vue-virtual-scroller 或自研实现
// 只渲染可视区域内的消息
```

### 3. 状态持久化策略

```typescript
// 使用 pinia-plugin-persistedstate
// 选择性持久化关键状态
const persistConfig = {
  authStore: {
    paths: ['token', 'refreshToken']
  },
  uiStore: {
    paths: ['theme', 'sidebarCollapsed']
  }
}
```

### 4. 缓存策略

| 数据类型 | 缓存方式 | 过期时间 |
|----------|----------|----------|
| 用户信息 | Memory | 会话期间 |
| 聊天室列表 | Memory + API | 5 分钟 |
| 消息历史 | Memory + IndexedDB | 1 小时 |
| 头像图片 | HTTP Cache | 1 天 |

## 安全考虑

### 1. XSS 防护

- 使用 Vue 的模板语法自动转义
- 消息内容使用 DOMPurify 净化
- 禁止渲染用户输入的 HTML

### 2. CSRF 防护

- 使用 SameSite Cookie
- 敏感操作添加验证码

### 3. Token 安全

- Access Token 存储在内存
- Refresh Token 存储在 httpOnly Cookie
- Token 过期自动刷新

## 扩展性设计

### 1. 插件系统

```typescript
// 插件接口定义
interface ChatPlugin {
  name: string
  install: (app: App, options?: Record<string, unknown>) => void
}

// 使用方式
app.use(EmojiPlugin)
app.use(FileUploadPlugin)
```

### 2. 主题系统

```typescript
// Naive UI 主题定制
const themeOverrides = {
  common: {
    primaryColor: '#07C160',  // 微信绿
    primaryColorHover: '#06AD56',
    primaryColorPressed: '#05944F',
  }
}
```

### 3. 国际化准备

```typescript
// 预留国际化结构
const messages = {
  zh: { /* 中文 */ },
  en: { /* 英文 */ }
}
// 使用 vue-i18n（后续添加）
```

## 监控与调试

### 1. 日志系统

```typescript
// 分级日志
logger.debug('调试信息')
logger.info('普通信息')
logger.warn('警告信息')
logger.error('错误信息')
```

### 2. 性能监控

```typescript
// 关键指标监控
- 首屏加载时间
- 消息发送延迟
- WebSocket 重连次数
- 虚拟滚动性能
```

### 3. 错误上报

```typescript
// 全局错误捕获
window.onerror = (message, source, lineno, colno, error) => {
  // 上报错误到监控服务
}
```
