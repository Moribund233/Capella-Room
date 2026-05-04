# Debug工具开发规划

> 本文档描述将原debug端工具迁移到user端（后续重命名为debug端）的开发规划

## 概述

原debug端包含的WebSocket测试、多用户认证等工具具有重要价值，但代码实现与现有SeredeliUI规范和后端API不完全匹配。本规划遵循以下原则：

- **遵循SeredeliUI规范** - 使用配置驱动、组合式函数架构
- **参考最新API文档** - 确保与后端API完全对齐
- **重写而非迁移** - 按新规范重新实现，不复制旧代码
- **保留核心功能** - 批量认证、WebSocket测试、API测试

## 设计规范

### 页面开发模式

- **简单单页**：直接编辑主视图（`/views`）
- **复杂单页**：主视图（`/views`）+ 组件模式（`/components`）
- **带子路由**：主视图（`/views`）+ 子页面（`/pages`）（复杂子页面再使用组件（`/components`））

### QuickBar集成

Debug工具通过DockBar/QuickBar集成到主界面，采用**工厂模式 + 组合式函数**架构。

## 开发阶段

### 阶段一：基础架构搭建

**目标**：建立Debug工具基础架构，配置路由和菜单

#### 任务清单

- [x] **1.1 创建目录结构**
  - `src/pages/debug/` - Debug工具子页面
  - `src/components/test/` - 测试相关组件
  - `src/composables/test/` - 测试相关组合式函数
  - `src/api/test.ts` - 测试专用API

- [x] **1.2 配置路由**
  - 添加Debug工具主路由 `/debug`
  - 配置子路由：`/debug/multi-user`, `/debug/websocket`, `/debug/api`

- [x] **1.3 配置DockBar**
  - 在UI配置中添加Debug工具入口
  - 配置子菜单项

- [x] **1.4 创建主视图框架**
  - `DebugView.vue` - Debug工具主视图（带子路由出口）

#### 验收标准
- [x] 可以通过导航访问Debug工具页面
- [x] 子路由切换正常
- [x] DockBar显示Debug工具入口

---

### 阶段二：多用户认证测试工具

**设计模式**：复杂单页（主视图 + 组件）

**参考API**：
- `POST /api/v1/auth/register` - 用户注册
- `POST /api/v1/auth/login` - 用户登录
- `POST /api/v1/auth/refresh` - Token刷新

**页面结构**：
```
src/
├── pages/
│   └── debug/
│       └── MultiUserTest.vue            # 多用户测试主页面
├── components/
│   └── test/
│       ├── UserBatchCard.vue            # 批量用户创建卡片
│       ├── UserListCard.vue             # 用户列表卡片
│       ├── UserDetailModal.vue          # 用户详情弹窗
│       └── ConnectionStatsCard.vue      # 连接统计卡片
├── composables/
│   └── test/
│       └── useMultiUser.ts              # 多用户管理逻辑
└── api/
    └── test.ts                          # 测试专用API
```

**功能需求**：
- 批量创建测试用户（自动注册）
- 批量登录获取Token
- 用户状态管理（在线/离线）
- 一键连接/断开所有用户WebSocket
- 用户列表展示（用户名、状态、Token有效期）
- 单个用户操作（登录、登出、刷新Token）

**数据存储**：
- 使用Pinia Store管理测试用户状态
- 使用sessionStorage持久化（避免干扰主应用localStorage）

**UI设计**：
- 顶部：批量操作工具栏（创建、登录、连接WS、清空）
- 中部：用户卡片网格（3列响应式）
- 底部：连接状态统计

#### 阶段2完成总结

**已完成内容**：
- ✅ 测试专用API模块 (`src/api/test.ts`)
- ✅ 多用户测试Store (`src/store/testUsers.ts`)
- ✅ 多用户组合式函数 (`src/composables/test/useMultiUser.ts`)
- ✅ 批量操作工具栏组件 (`src/components/test/BatchOperationToolbar.vue`)
- ✅ 连接统计卡片组件 (`src/components/test/ConnectionStatsCard.vue`)
- ✅ 用户卡片组件 (`src/components/test/UserCard.vue`)
- ✅ 多用户测试主页面 (`src/pages/debug/MultiUserTest.vue`)
- ✅ 移动端响应式适配
- ✅ 导出/导入用户凭据功能

**功能特性**：
- 批量创建测试用户（自动注册）
- 批量登录获取Token
- 用户状态管理（在线/离线/WS连接）
- 一键连接/断开所有用户WebSocket
- 批量刷新Token
- 批量登出
- 单个用户操作（登录、登出、刷新Token、连接/断开WS、删除）
- 导出用户凭据为JSON文件
- 导入用户凭据批量创建用户
- 响应式设计（移动端/桌面端适配）

---

### 阶段三：WebSocket测试工具 ✅ 已完成

**设计模式**：复杂单页（主视图 + 子组件）

**参考API**：
- WebSocket连接：`ws://localhost:8080/ws`
- 消息类型：`ChatMessage`, `Typing`, `MessageRead`, `EditMessage`, `DeleteMessage`
- 详见：`docs/api/v1/websocket/message.md`

**页面结构**：
```
src/
├── pages/
│   └── debug/
│       └── WebSocketTest.vue            # WebSocket测试主页面
├── components/
│   └── test/
│       ├── LatencyTestCard.vue          # 延迟测试卡片
│       ├── StabilityTestCard.vue        # 稳定性测试卡片
│       ├── StressTestCard.vue           # 压力测试卡片
│       └── WsLogPanel.vue               # WebSocket日志面板
├── composables/
│   └── test/
│       └── useWsTest.ts                 # WebSocket测试逻辑
└── store/
    └── wsTest.ts                        # WebSocket测试状态管理
```

**功能需求**：

#### 3.1 延迟测试 ✅
- 发送Ping消息，测量往返延迟
- 显示延迟统计（最小、最大、平均、P99）
- 延迟趋势图表
- 支持批量测试（多用户同时Ping）

#### 3.2 稳定性测试 ✅
- 长时间保持连接（可配置时长）
- 自动心跳检测
- 记录断连次数和原因
- 生成稳定性报告

#### 3.3 压力测试 ✅
- 多用户并发发送消息
- 可配置并发数、消息数、发送间隔
- 实时显示发送速率、成功率
- 服务器响应时间统计

#### 3.4 消息类型测试 ✅（集成到聊天页面）
- 测试各种消息类型的发送和接收
- 支持自定义消息内容
- 查看消息响应

**UI设计**：
- 顶部：连接状态概览（提示用户前往多用户页面管理连接）
- 中部：标签页切换不同测试类型（延迟、稳定性、压力）
- 底部：WebSocket消息日志（可折叠）

#### 阶段3完成总结

**已完成内容**：
- ✅ WebSocket测试Store (`src/store/wsTest.ts`)
- ✅ WebSocket测试组合式函数 (`src/composables/test/useWsTest.ts`)
- ✅ 延迟测试卡片 (`src/components/test/LatencyTestCard.vue`)
- ✅ 稳定性测试卡片 (`src/components/test/StabilityTestCard.vue`)
- ✅ 压力测试卡片 (`src/components/test/StressTestCard.vue`)
- ✅ WebSocket日志面板 (`src/components/test/WsLogPanel.vue`)
- ✅ WebSocket测试主页面 (`src/pages/debug/WebSocketTest.vue`)
- ✅ 聊天页面测试面板 (`src/components/test/ChatTestPanel.vue`)

**功能特性**：
- 延迟测试（Ping/Pong往返时间测量，统计指标）
- 稳定性测试（长时间连接保持，断连统计）
- 压力测试（并发消息发送，成功率统计）
- 测试日志记录和导出
- 与多用户页面集成（连接管理）
- 聊天页面集成测试控制面板

**调整说明**：
- WebSocket连接管理移至多用户页面，保持单一职责原则
- 消息类型测试集成到聊天页面，更贴近实际使用场景
- 测试面板作为浮层显示在聊天卡片内部，不挤压原有布局

---

### 阶段四：API测试工具

**设计模式**：简单单页

**参考API**：
- `docs/api/v1/http/rooms.md` - 房间API
- `docs/api/v1/http/messages.md` - 消息API
- `docs/api/v1/http/user.md` - 用户API

**页面结构**：
```
src/
├── pages/
│   └── debug/
│       └── ApiTest.vue                  # API测试主页面
├── components/
│   └── test/
│       ├── ApiRequestPanel.vue          # 请求面板
│       ├── ApiResponsePanel.vue         # 响应面板
│       ├── ApiHistoryPanel.vue          # 历史记录面板
│       └── ApiEndpointSelect.vue        # API端点选择器
└── composables/
    └── test/
        └── useApiTest.ts                # API测试逻辑
```

**功能需求**：
- 快速测试HTTP API端点
- 预设常用API模板（房间、消息、用户相关）
- 自定义请求头和请求体
- 请求历史记录（保存最近20条）
- 响应格式化显示（JSON高亮）
- 响应时间统计

**预设API端点**：
```typescript
const apiEndpoints = [
  // 房间API
  { label: '获取房间列表', method: 'GET', path: '/rooms', category: '房间' },
  { label: '创建房间', method: 'POST', path: '/rooms', category: '房间' },
  { label: '获取房间详情', method: 'GET', path: '/rooms/:id', category: '房间' },
  { label: '加入房间', method: 'POST', path: '/rooms/:id/join', category: '房间' },
  { label: '离开房间', method: 'POST', path: '/rooms/:id/leave', category: '房间' },
  
  // 消息API
  { label: '获取房间消息', method: 'GET', path: '/rooms/:room_id/messages', category: '消息' },
  { label: '发送消息', method: 'POST', path: '/rooms/:room_id/messages', category: '消息' },
  { label: '搜索消息', method: 'GET', path: '/messages/search', category: '消息' },
  
  // 用户API
  { label: '获取当前用户', method: 'GET', path: '/users/me', category: '用户' },
  { label: '获取用户列表', method: 'GET', path: '/users', category: '用户' },
  { label: '更新用户信息', method: 'PUT', path: '/users/me', category: '用户' },
  
  // 认证API
  { label: '用户注册', method: 'POST', path: '/auth/register', category: '认证' },
  { label: '用户登录', method: 'POST', path: '/auth/login', category: '认证' },
  { label: '刷新Token', method: 'POST', path: '/auth/refresh', category: '认证' },
]
```

**UI设计**：
- 左侧：API端点选择器（按分类分组）
- 右上：请求面板（方法、路径、请求体）
- 右下：响应面板（状态码、响应体、耗时）
- 底部：请求历史记录

---

## 路由配置

```typescript
// src/router/routes.ts
{
  path: 'debug',
  name: 'Debug',
  component: () => import('@/views/DebugView.vue'),
  redirect: '/debug/multi-user',
  meta: {
    title: '调试工具',
    requiresAuth: true,
    icon: 'Bug',
  },
  children: [
    {
      path: 'multi-user',
      name: 'MultiUserTest',
      component: () => import('@/pages/debug/MultiUserTest.vue'),
      meta: { title: '多用户测试', icon: 'Users' },
    },
    {
      path: 'websocket',
      name: 'WebSocketTest',
      component: () => import('@/pages/debug/WebSocketTest.vue'),
      meta: { title: 'WebSocket测试', icon: 'Wifi' },
    },
    {
      path: 'api',
      name: 'ApiTest',
      component: () => import('@/pages/debug/ApiTest.vue'),
      meta: { title: 'API测试', icon: 'Terminal' },
    },
  ],
}
```

---

## DockBar配置

```typescript
// src/config/ui.ts
export const dockBarConfig: DockItemConfig[] = [
  // ... 其他配置
  {
    key: 'debug',
    enabled: true,
    position: 'bottom',
    offset: 24,
    items: [
      { key: 'multi-user', label: '多用户测试', icon: 'Users', path: '/debug/multi-user' },
      { key: 'websocket', label: 'WebSocket测试', icon: 'Wifi', path: '/debug/websocket' },
      { key: 'api', label: 'API测试', icon: 'Terminal', path: '/debug/api' },
    ],
  },
]
```

---

## 开发优先级建议

1. **高优先级**
   - 阶段一：基础架构搭建（必须先完成）
   - 阶段二：多用户认证测试（核心功能）

2. **中优先级**
   - 阶段四：API测试工具（使用频率高）

3. **低优先级**
   - 阶段三：WebSocket测试工具（功能复杂，可后续完善）

---

## 参考文档

- [页面开发指南](./page-development.md) - SeredeliUI页面开发规范
- [QuickBar开发指南](./quickbar-development.md) - Dock栏配置说明
- [后端集成指南](./backend-integration.md) - API集成说明
- [API文档](../../docs/api/v1/) - 后端API文档

---

## 注意事项

1. **权限控制**：Debug工具仅管理员可用，路由需添加 `requiresAuth: true`
2. **数据隔离**：测试用户使用独立存储（sessionStorage），避免干扰主应用
3. **性能考虑**：批量操作时注意性能，避免阻塞UI
4. **错误处理**：完善的错误提示和日志记录
5. **响应式设计**：适配移动端和桌面端

---

*文档创建时间: 2026-05-04*
*关联任务: debug端工具迁移*
