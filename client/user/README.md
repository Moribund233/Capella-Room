# Capella Room - 用户端

Capella Room 用户端是一个现代化的实时聊天室客户端，提供仿微信的聊天体验，支持响应式三端适配（桌面、平板、手机）。

## 功能特性

### 核心功能
- 用户认证（登录、注册、Token 自动刷新）
- 实时聊天（WebSocket 连接，支持断线重连）
- 聊天室管理（创建、加入、离开、搜索）
- 消息功能（发送、接收、回复、编辑、删除、搜索）
- 个人中心（资料管理、头像上传、统计信息）
- 系统设置（主题、通知、隐私、安全、语言等）

### 技术特性
- 响应式设计（桌面 >1024px、平板 768-1024px、手机 <768px）
- 流畅动画（页面切换、消息气泡、手势反馈）
- 性能优化（路由懒加载、防抖节流、骨架屏）
- 错误处理（全局错误捕获、错误边界）

## 技术栈

| 类别 | 技术 | 版本 |
|------|------|------|
| 框架 | Vue | ^3.5.32 |
| 路由 | Vue Router | ^5.0.4 |
| 状态管理 | Pinia | ^3.0.4 |
| UI 组件库 | Naive UI | ^2.44.1 |
| 图标库 | Lucide Vue | latest |
| 构建工具 | Vite | ^8.0.8 |
| 类型系统 | TypeScript | ~6.0.0 |

## 开发环境

### 前置要求
- Node.js: ^20.19.0 || >=22.12.0
- pnpm: >=8.0.0

### 安装依赖
```bash
pnpm install
```

### 开发服务器
```bash
pnpm dev
```

### 代码检查
```bash
# 类型检查
pnpm type-check

# 代码规范检查
pnpm lint

# 代码格式化
pnpm format
```

### 生产构建
```bash
pnpm build
```

## 项目结构

```
src/
├── api/              # API 接口封装
├── components/       # 组件
│   ├── chat/         # 聊天相关
│   ├── common/       # 通用组件
│   ├── error/        # 错误处理
│   ├── layout/       # 布局组件
│   ├── message/      # 消息组件
│   ├── profile/      # 个人中心
│   ├── quick/        # 快捷操作
│   ├── room/         # 聊天室
│   ├── settings/     # 设置
│   └── ui/           # UI 组件（骨架屏、空状态等）
├── composables/      # 组合式函数
├── constants/        # 常量定义
├── layouts/          # 布局模板
├── router/           # 路由配置
├── services/         # 服务（WebSocket、HTTP、错误处理）
├── stores/           # Pinia 状态管理
├── styles/           # 全局样式
├── types/            # TypeScript 类型
├── utils/            # 工具函数
└── views/            # 页面视图
```

## 开发规范

### 代码规范
- 使用 ESLint + Prettier + Oxlint 进行代码检查
- 遵循 Vue 3.3+ 组合式 API 最佳实践
- 组件名使用 PascalCase，多单词命名
- 组合式函数使用 `useXxx` 命名

### 类型规范
- 全项目 TypeScript 严格模式
- API 响应类型统一定义
- 组件 Props 必须定义类型

### 注释规范
- 函数级注释说明功能、参数、返回值
- 复杂逻辑添加行内注释
- 组件说明文档写在 JSDoc 中

## 环境变量

复制 `.env.example` 为 `.env` 并配置：

```env
# API 基础地址
VITE_API_BASE_URL=http://localhost:8080/api/v1

# WebSocket 地址
VITE_WS_URL=ws://localhost:8080/ws
```

## 浏览器支持

- Chrome >= 90
- Firefox >= 88
- Safari >= 14
- Edge >= 90

## 相关文档

- [项目概述](./docs/overview.md)
- [开发阶段规划](./docs/phases.md)
- [架构设计](./docs/architecture.md)
- [API 集成规范](./docs/api-integration.md)
- [部署文档](./docs/deployment.md)

## 许可证

MIT License
