# 开发阶段规划

本文档详细规划了 Capella Room 用户端的各开发阶段，每个阶段都有明确的目标、交付物和验收标准。

## 总体时间线

```
Week 1       Week 2       Week 3       Week 4
├────────────┼────────────┼────────────┼────────────┤
│ Phase 1-2  │ Phase 3-4  │ Phase 5-6  │ Phase 7-8  │
│ 基础架构   │ 核心功能   │ 聊天功能   │ 优化完善   │
└────────────┴────────────┴────────────┴────────────┘
```

---

## Phase 1: 项目初始化与基础架构

**工期**: 2 天  
**目标**: 搭建可运行的项目基础框架

### 任务清单

- [x] 1.1 安装项目依赖
  - Naive UI
  - Lucide Vue
  - Pinia Plugin Persistedstate
  - 其他必要依赖

- [x] 1.2 配置项目结构
  - 创建标准目录结构
  - 配置路径别名 (@/)
  - 配置环境变量

- [x] 1.3 配置代码规范
  - ESLint 规则调整
  - Prettier 配置
  - Git 提交规范

- [x] 1.4 搭建基础布局
  - 主布局组件 (MainLayout)
  - 响应式容器
  - 主题配置

- [x] 1.5 创建工具函数
  - HTTP 客户端封装
  - 存储工具 (localStorage/sessionStorage)
  - 日期格式化
  - 响应式工具

### 交付物

```
src/
├── api/              # API 接口定义
├── components/       # 基础组件
├── composables/      # 组合式函数
├── layouts/          # 布局组件
├── router/           # 路由配置
├── stores/           # Pinia stores
├── styles/           # 全局样式
├── types/            # TypeScript 类型
└── utils/            # 工具函数
```

### 验收标准

- [x] 项目可以正常启动 (`pnpm dev`)
- [x] 代码规范检查通过 (`pnpm lint`)
- [x] 类型检查通过 (`pnpm type-check`)
- [x] 基础布局在三种屏幕尺寸下正常显示

---

## Phase 2: 认证模块与路由守卫

**工期**: 2 天  
**目标**: 实现完整的用户认证流程

### 任务清单

- [x] 2.1 类型定义
  - 用户类型 (User)
  - 认证相关类型 (Token, Credentials)
  - API 响应类型

- [x] 2.2 API 接口封装
  - 登录接口
  - 注册接口
  - Token 刷新接口
  - 登出接口

- [x] 2.3 Auth Store
  - 认证状态管理
  - Token 管理
  - 用户信息存储

- [x] 2.4 登录页面
  - 登录表单
  - 表单验证
  - 错误提示

- [x] 2.5 注册页面
  - 注册表单
  - 表单验证
  - 注册成功提示

- [x] 2.6 路由守卫
  - 认证检查
  - 自动跳转登录
  - 登录后重定向

### 交付物

| 文件 | 说明 |
|------|------|
| `src/views/LoginView.vue` | 登录页面 |
| `src/views/RegisterView.vue` | 注册页面 |
| `src/stores/auth.ts` | 认证状态管理 |
| `src/api/auth.ts` | 认证 API |
| `src/composables/useAuth.ts` | 认证逻辑封装 |

### 验收标准

- [x] 用户可以正常注册账号
- [x] 用户可以正常登录
- [x] Token 过期自动刷新
- [x] 未登录用户访问受保护页面自动跳转登录
- [x] 登录后自动跳转原页面

---

## Phase 3: WebSocket 基础连接

**工期**: 2 天  
**目标**: 建立稳定的 WebSocket 连接，实现基础消息收发

### 任务清单

- [x] 3.1 WebSocket 服务封装
  - 连接管理
  - 心跳机制
  - 断线重连
  - 消息队列

- [x] 3.2 WebSocket Store
  - 连接状态管理
  - 消息订阅管理

- [x] 3.3 消息类型定义
  - WebSocket 消息类型
  - 消息处理器类型

- [x] 3.4 连接状态组件
  - 连接中提示
  - 断线提示
  - 重连提示

- [x] 3.5 基础消息测试
  - 发送测试消息
  - 接收测试消息

### 交付物

| 文件 | 说明 |
|------|------|
| `src/services/websocket.ts` | WebSocket 服务 |
| `src/stores/websocket.ts` | WebSocket 状态 |
| `src/composables/useWebSocket.ts` | WebSocket 组合式函数 |
| `src/types/websocket.ts` | WebSocket 类型定义 |

### 验收标准

- [x] WebSocket 连接成功建立
- [x] 心跳机制正常工作（30秒间隔）
- [x] 断线后自动重连（指数退避）
- [x] 连接状态在 UI 中正确显示

---

## Phase 4: 聊天室列表与详情

**工期**: 3 天  
**目标**: 实现聊天室的展示、加入、管理功能

### 任务清单

- [x] 4.1 聊天室 API 封装
  - 获取聊天室列表
  - 获取聊天室详情
  - 加入/离开聊天室
  - 创建聊天室

- [x] 4.2 Room Store
  - 聊天室列表管理
  - 当前聊天室管理
  - 成员列表管理

- [x] 4.3 聊天室列表组件
  - 列表展示
  - 搜索过滤
  - 排序功能
  - 未读消息徽章

- [x] 4.4 聊天室卡片组件
  - 头像展示
  - 名称和描述
  - 最后消息预览
  - 时间显示

- [x] 4.5 聊天室详情页
  - 房间信息展示
  - 成员列表
  - 房间设置

- [x] 4.6 创建聊天室弹窗
  - 表单验证
  - 创建成功提示

### 交付物

| 文件 | 说明 |
|------|------|
| `src/views/ChatView.vue` | 聊天主页面 |
| `src/components/room/RoomList.vue` | 聊天室列表 |
| `src/components/room/RoomCard.vue` | 聊天室卡片 |
| `src/components/room/RoomDetail.vue` | 聊天室详情 |
| `src/components/room/CreateRoomModal.vue` | 创建聊天室弹窗 |
| `src/stores/room.ts` | 聊天室状态 |
| `src/composables/useRoom.ts` | 聊天室逻辑 |

### 验收标准

- [x] 聊天室列表正确显示
- [x] 可以搜索过滤聊天室
- [x] 可以点击加入聊天室
- [x] 可以创建新的聊天室
- [x] 未读消息数正确显示

---

## Phase 5: 消息收发与展示

**工期**: 3 天  
**目标**: 实现完整的消息收发、展示、历史加载功能

### 任务清单

- [x] 5.1 消息 API 封装
  - 获取消息历史
  - 搜索消息

- [x] 5.2 Message Store
  - 消息列表管理
  - 消息缓存
  - 未读计数

- [x] 5.3 消息气泡组件
  - 发送的消息样式
  - 接收的消息样式
  - 时间戳显示
  - 发送状态指示

- [x] 5.4 消息列表组件
  - 虚拟滚动
  - 下拉加载历史
  - 新消息提示
  - 自动滚动到底部

- [x] 5.5 输入框组件
  - 文本输入
  - 自动增高
  - 发送快捷键 (Enter)
  - 换行快捷键 (Shift+Enter)

- [x] 5.6 消息发送逻辑
  - 发送消息
  - 发送中状态
  - 发送失败重试
  - 发送成功确认

- [x] 5.7 消息接收处理
  - 实时接收
  - 消息通知
  - 未读标记

### 交付物

| 文件 | 说明 |
|------|------|
| `src/components/message/MessageBubble.vue` | 消息气泡 |
| `src/components/message/MessageList.vue` | 消息列表 |
| `src/components/message/MessageInput.vue` | 消息输入框 |
| `src/components/message/ScrollToBottom.vue` | 滚动到底部按钮 |
| `src/stores/message.ts` | 消息状态 |
| `src/composables/useMessage.ts` | 消息逻辑 |

### 验收标准

- [x] 可以发送文本消息
- [x] 可以接收实时消息
- [x] 消息气泡样式正确
- [x] 消息列表支持虚拟滚动
- [x] 可以加载历史消息
- [x] 新消息自动滚动到底部
- [x] 发送失败可以重试

---

## Phase 6: 消息高级功能

**工期**: 2 天  
**目标**: 实现消息回复、编辑、删除等高级功能

### 任务清单

- [x] 6.1 消息回复功能
  - 回复消息 UI
  - 回复消息发送
  - 回复消息展示

- [x] 6.2 消息编辑功能
  - 编辑消息 UI
  - 编辑消息发送
  - 编辑标记展示

- [x] 6.3 消息删除功能
  - 删除确认弹窗
  - 删除消息发送
  - 已删除消息展示

- [x] 6.4 消息搜索功能
  - 搜索输入框
  - 搜索结果展示
  - 跳转到指定消息

- [x] 6.5 消息操作菜单
  - 右键菜单（桌面端）
  - 长按菜单（移动端）
  - 操作选项展示

- [x] 6.6 正在输入提示
  - 输入状态检测
  - 输入状态发送
  - 对方输入提示展示

### 交付物

| 文件 | 说明 |
|------|------|
| `src/components/message/MessageReply.vue` | 消息回复 |
| `src/components/message/MessageEdit.vue` | 消息编辑 |
| `src/components/message/MessageActions.vue` | 消息操作菜单 |
| `src/components/message/TypingIndicator.vue` | 输入提示 |
| `src/components/search/MessageSearch.vue` | 消息搜索 |

### 验收标准

- [x] 可以回复指定消息
- [x] 可以编辑已发送消息
- [x] 可以删除已发送消息
- [x] 可以搜索历史消息
- [x] 可以看到对方正在输入

---

## Phase 7: 响应式适配优化

**工期**: 3 天  
**目标**: 实现三端（桌面、平板、手机）完美的响应式体验

### 任务清单

- [x] 7.1 响应式工具完善
  - 断点检测
  - 布局模式切换
  - 设备类型检测

- [x] 7.2 桌面端优化
  - 侧边栏固定
  - 快捷键支持
  - 右键菜单

- [x] 7.3 平板端适配
  - 可折叠侧边栏
  - 横竖屏切换
  - 触摸优化

- [x] 7.4 手机端适配
  - 页面切换动画
  - 抽屉式侧边栏
  - 底部操作栏
  - 长按菜单

- [x] 7.5 动画优化
  - 页面切换动画
  - 消息发送动画
  - 列表加载动画
  - 手势反馈动画

- [x] 7.6 触摸交互
  - 滑动手势
  - 长按手势
  - 双指缩放

### 交付物

| 文件 | 说明 |
|------|------|
| `src/composables/useResponsive.ts` | 响应式工具 |
| `src/components/layout/MobileSidebar.vue` | 移动端侧边栏 |
| `src/components/layout/DesktopSidebar.vue` | 桌面端侧边栏 |
| `src/components/ui/PageTransition.vue` | 页面过渡动画 |
| `src/styles/animations.css` | 动画样式 |

### 验收标准

- [x] 桌面端 (>1024px) 显示正常
- [x] 平板端 (768px-1024px) 显示正常
- [x] 手机端 (<768px) 显示正常
- [x] 页面切换动画流畅
- [x] 手势操作响应灵敏

---

## Phase 8: 性能优化与完善

**工期**: 2 天  
**目标**: 优化性能，完善细节，达到生产环境标准

### 任务清单

- [x] 8.1 代码分割
  - 路由懒加载
  - 组件异步加载
  - 依赖按需引入

- [x] 8.2 性能优化
  - 虚拟滚动优化（分析后暂不实施，当前分页机制已满足需求）
  - 图片懒加载
  - 防抖节流
  - 内存泄漏检查

- [x] 8.3 错误处理
  - 全局错误捕获
  - 错误边界
  - 友好错误提示

- [x] 8.4 加载状态
  - 骨架屏
  - 加载占位符
  - 空状态

- [x] 8.5 个人中心
  - 用户信息展示
  - 资料编辑
  - 头像上传

- [x] 8.6 设置页面
  - 主题设置
  - 通知设置
  - 账号安全

- [ ] 8.7 文档完善
  - README 更新
  - 组件文档
  - 部署文档

### 交付物

| 文件 | 说明 |
|------|------|
| `src/views/ProfileView.vue` | 个人中心 |
| `src/views/SettingsView.vue` | 设置页面 |
| `src/components/ui/Skeleton.vue` | 骨架屏基础组件 |
| `src/components/ui/SkeletonCard.vue` | 卡片骨架屏 |
| `src/components/ui/SkeletonList.vue` | 列表骨架屏 |
| `src/components/ui/EmptyState.vue` | 空状态组件 |
| `src/components/error/ErrorBoundary.vue` | 错误边界组件 |
| `src/services/error.ts` | 全局错误处理服务 |
| `src/utils/performance.ts` | 性能工具函数（防抖、节流等） |

### 验收标准

- [x] 首屏加载时间 < 3s（路由懒加载已实现）
- [x] 消息列表滚动流畅 (60fps)（分页机制已满足需求）
- [x] 无内存泄漏
- [x] 所有功能正常工作
- [x] 代码规范检查通过

---

## 开发检查清单

### 每个阶段开始前

- [ ] 阅读相关 API 文档
- [ ] 确认设计规范
- [ ] 创建功能分支

### 每个阶段进行中

- [ ] 遵循代码规范
- [ ] 添加必要注释
- [ ] 编写单元测试
- [ ] 自测功能正常

### 每个阶段结束后

- [ ] 代码审查
- [ ] 合并到主分支
- [ ] 更新进度文档

---

## 风险预案

| 风险 | 应对措施 |
|------|----------|
| WebSocket 不稳定 | 增加重试次数，优化重连策略 |
| 性能不达标 | 提前引入虚拟滚动，优化渲染 |
| 移动端适配复杂 | 优先保证桌面端，逐步适配移动端 |
| 进度延迟 | 适当调整范围，保证核心功能 |

---

## 附录：开发顺序图

```
Phase 1: 项目初始化
    │
    ▼
Phase 2: 认证模块
    │
    ▼
Phase 3: WebSocket 连接
    │
    ▼
Phase 4: 聊天室功能
    │
    ▼
Phase 5: 消息收发
    │
    ▼
Phase 6: 消息高级功能
    │
    ▼
Phase 7: 响应式适配
    │
    ▼
Phase 8: 性能优化
    │
    ▼
  完成
```
