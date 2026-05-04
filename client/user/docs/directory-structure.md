# 目录结构规范

本文档定义了项目的目录结构规范，确保代码组织清晰、可维护、易扩展。

## 总体结构

```
client/user/
├── docs/                       # 开发文档
│   ├── overview.md            # 项目概述
│   ├── architecture.md        # 技术架构
│   ├── design-system.md       # UI 设计规范
│   ├── phases.md              # 开发阶段规划
│   ├── api-integration.md     # API 集成规范
│   └── directory-structure.md # 本文档
│
├── public/                     # 静态资源
│   └── favicon.ico
│
├── src/
│   ├── api/                    # API 接口模块
│   ├── assets/                 # 资源文件
│   ├── components/             # 组件
│   ├── composables/            # 组合式函数
│   ├── constants/              # 常量定义
│   ├── layouts/                # 布局组件
│   ├── router/                 # 路由配置
│   ├── services/               # 服务层
│   ├── stores/                 # Pinia 状态管理
│   ├── styles/                 # 全局样式
│   ├── types/                  # TypeScript 类型
│   ├── utils/                  # 工具函数
│   └── views/                  # 页面视图
│
├── .env                        # 环境变量
├── .env.development            # 开发环境变量
├── .env.production             # 生产环境变量
├── index.html                  # HTML 入口
├── package.json                # 项目配置
├── tsconfig.json               # TypeScript 配置
└── vite.config.ts              # Vite 配置
```

## 详细说明

### src/api/ - API 接口模块

按功能模块组织 API 接口。

```
api/
├── auth.ts          # 认证相关 API
├── user.ts          # 用户相关 API
├── room.ts          # 聊天室相关 API
└── message.ts       # 消息相关 API
```

**规范**：
- 每个文件对应一个功能模块
- 使用对象方式导出 API 方法
- 每个方法必须添加 JSDoc 注释

### src/assets/ - 资源文件

```
assets/
├── images/          # 图片资源
│   ├── logo.png
│   └── default-avatar.png
├── icons/           # 图标资源
└── styles/          # 样式资源（非全局）
```

### src/components/ - 组件

采用原子设计方法论组织组件。

```
components/
├── base/                    # 基础组件 (Atoms)
│   ├── Button/
│   │   ├── Button.vue
│   │   └── types.ts
│   ├── Input/
│   ├── Avatar/
│   └── index.ts             # 统一导出
│
├── ui/                      # UI 组件 (Molecules)
│   ├── Skeleton/
│   ├── EmptyState/
│   ├── Loading/
│   └── Modal/
│
├── room/                    # 聊天室相关组件 (Organisms)
│   ├── RoomList/
│   ├── RoomCard/
│   ├── RoomDetail/
│   └── CreateRoomModal/
│
├── message/                 # 消息相关组件 (Organisms)
│   ├── MessageBubble/
│   ├── MessageList/
│   ├── MessageInput/
│   ├── MessageReply/
│   └── MessageActions/
│
├── layout/                  # 布局相关组件
│   ├── Sidebar/
│   ├── Header/
│   ├── Footer/
│   └── PageTransition/
│
└── error/                   # 错误相关组件
    ├── ErrorBoundary/
    └── ErrorFallback/
```

**组件文件规范**：

```
ComponentName/
├── ComponentName.vue        # 主组件文件
├── ComponentName.spec.ts    # 测试文件（可选）
├── types.ts                 # 组件类型定义（可选）
├── composables.ts           # 组件专用 composables（可选）
└── index.ts                 # 导出文件
```

**组件命名规范**：
- 使用 PascalCase
- 语义化命名，避免缩写
- 文件夹名与组件名一致

### src/composables/ - 组合式函数

```
composables/
├── useAuth.ts               # 认证逻辑
├── useWebSocket.ts          # WebSocket 连接
├── useRoom.ts               # 聊天室逻辑
├── useMessage.ts            # 消息逻辑
├── useUser.ts               # 用户逻辑
├── useResponsive.ts         # 响应式适配
├── useNotification.ts       # 通知管理
├── useErrorHandler.ts       # 错误处理
└── index.ts                 # 统一导出
```

**命名规范**：
- 使用 `use` 前缀
- 使用 camelCase
- 功能单一，职责清晰

### src/constants/ - 常量定义

```
constants/
├── index.ts                 # 统一导出
├── errorCodes.ts            # 错误码
├── storageKeys.ts           # 存储键名
├── routes.ts                # 路由常量
└── websocket.ts             # WebSocket 常量
```

### src/layouts/ - 布局组件

```
layouts/
├── MainLayout.vue           # 主布局
├── AuthLayout.vue           # 认证页面布局
└── index.ts                 # 统一导出
```

### src/router/ - 路由配置

```
router/
├── index.ts                 # 路由入口
├── routes.ts                # 路由定义
├── guards.ts                # 路由守卫
└── types.ts                 # 路由类型扩展
```

### src/services/ - 服务层

```
services/
├── http.ts                  # HTTP 客户端
├── websocket.ts             # WebSocket 服务
└── storage.ts               # 存储服务
```

### src/stores/ - Pinia 状态管理

```
stores/
├── index.ts                 # 统一导出
├── auth.ts                  # 认证状态
├── user.ts                  # 用户状态
├── room.ts                  # 聊天室状态
├── message.ts               # 消息状态
├── websocket.ts             # WebSocket 状态
└── ui.ts                    # UI 状态
```

**Store 规范**：
- 使用函数式定义（Setup Store）
- 命名使用 camelCase
- 导出时使用 `useXxxStore` 格式

### src/styles/ - 全局样式

```
styles/
├── index.css                # 样式入口
├── variables.css            # CSS 变量
├── animations.css           # 动画样式
├── utilities.css            # 工具类
└── naive-ui-overrides.css   # Naive UI 样式覆盖
```

### src/types/ - TypeScript 类型

```
types/
├── index.ts                 # 统一导出
├── api.ts                   # API 响应类型
├── user.ts                  # 用户类型
├── room.ts                  # 聊天室类型
├── message.ts               # 消息类型
├── websocket.ts             # WebSocket 类型
└── router.ts                # 路由类型
```

### src/utils/ - 工具函数

```
utils/
├── index.ts                 # 统一导出
├── format.ts                # 格式化工具
├── date.ts                  # 日期工具
├── validate.ts              # 验证工具
├── storage.ts               # 存储工具
├── dom.ts                   # DOM 工具
└── crypto.ts                # 加密工具
```

**工具函数规范**：
- 纯函数，无副作用
- 添加 JSDoc 注释
- 添加单元测试

### src/views/ - 页面视图

```
views/
├── LoginView.vue            # 登录页面
├── RegisterView.vue         # 注册页面
├── ChatView.vue             # 聊天主页面
├── ProfileView.vue          # 个人中心
├── SettingsView.vue         # 设置页面
└── NotFoundView.vue         # 404 页面
```

**页面组件规范**：
- 使用 `View` 后缀
- 使用 PascalCase
- 只负责页面级逻辑，业务逻辑下沉到 composables

## 命名规范

### 文件命名

| 类型 | 规范 | 示例 |
|------|------|------|
| 组件 | PascalCase | `MessageBubble.vue` |
| 组合式函数 | camelCase | `useAuth.ts` |
| 工具函数 | camelCase | `formatDate.ts` |
| 常量文件 | camelCase | `errorCodes.ts` |
| 类型文件 | camelCase | `message.ts` |
| 样式文件 | kebab-case | `variables.css` |
| 测试文件 | camelCase | `useAuth.spec.ts` |

### 代码命名

| 类型 | 规范 | 示例 |
|------|------|------|
| 组件名 | PascalCase | `MessageBubble` |
| Props | camelCase | `messageId` |
| Emits | camelCase | `update:message` |
| 组合式函数 | camelCase | `useAuth` |
| Store | camelCase | `useAuthStore` |
| 类型 | PascalCase | `MessageType` |
| 接口 | PascalCase | `Message` |
| 枚举 | PascalCase | `WebSocketMessageType` |
| 常量 | UPPER_SNAKE_CASE | `MAX_RETRY_COUNT` |

## 导入规范

### 路径别名

```typescript
// vite.config.ts
export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
```

### 导入顺序

```typescript
// 1. Vue 核心
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'

// 2. 第三方库
import { useMessage } from 'naive-ui'
import { Send } from 'lucide-vue-next'

// 3. 内部模块
import { useAuthStore } from '@/stores'
import { useWebSocket } from '@/composables'
import { messageApi } from '@/api'

// 4. 类型导入
import type { Message } from '@/types'

// 5. 样式导入
import './styles.css'
```

## 代码组织示例

### 完整组件示例

```vue
<!-- src/components/message/MessageBubble/MessageBubble.vue -->
<script setup lang="ts">
/**
 * 消息气泡组件
 * 展示单条消息，支持发送和接收两种样式
 */

import { computed } from 'vue'
import { formatTime } from '@/utils'
import type { Message } from '@/types'
import UserAvatar from '@/components/base/UserAvatar/UserAvatar.vue'

interface Props {
  /** 消息数据 */
  message: Message
  /** 是否为自己发送的消息 */
  isSelf: boolean
  /** 是否显示头像 */
  showAvatar?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showAvatar: true
})

const emit = defineEmits<{
  /** 点击消息 */
  (e: 'click', message: Message): void
  /** 右键菜单 */
  (e: 'contextmenu', message: Message, event: MouseEvent): void
}>()

// 计算属性
const bubbleClass = computed(() => ({
  'message-bubble': true,
  'message-bubble--self': props.isSelf,
  'message-bubble--other': !props.isSelf
}))

const formattedTime = computed(() => formatTime(props.message.created_at))

// 方法
function handleClick() {
  emit('click', props.message)
}

function handleContextMenu(event: MouseEvent) {
  emit('contextmenu', props.message, event)
}
</script>

<template>
  <div :class="bubbleClass" @click="handleClick" @contextmenu.prevent="handleContextMenu">
    <UserAvatar
      v-if="showAvatar && !isSelf"
      :src="message.sender.avatar_url"
      :name="message.sender.username"
      size="sm"
    />
    
    <div class="message-bubble__content">
      <div v-if="!isSelf" class="message-bubble__sender">
        {{ message.sender.username }}
      </div>
      
      <div class="message-bubble__body">
        {{ message.content }}
      </div>
      
      <div class="message-bubble__time">
        {{ formattedTime }}
      </div>
    </div>
    
    <UserAvatar
      v-if="showAvatar && isSelf"
      :src="message.sender.avatar_url"
      :name="message.sender.username"
      size="sm"
    />
  </div>
</template>

<style scoped>
.message-bubble {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 16px;
}

.message-bubble--self {
  flex-direction: row-reverse;
}

.message-bubble__content {
  max-width: 70%;
}

.message-bubble__body {
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 14px;
  line-height: 22px;
}

.message-bubble--self .message-bubble__body {
  background-color: #95ec69;
  border-radius: 8px 2px 8px 8px;
}

.message-bubble--other .message-bubble__body {
  background-color: #ffffff;
  border-radius: 2px 8px 8px 8px;
}

.message-bubble__time {
  font-size: 12px;
  color: #8c8c8c;
  margin-top: 4px;
}
</style>
```

## 扩展建议

### 新增功能模块时

1. 在 `src/api/` 添加 API 接口
2. 在 `src/types/` 添加类型定义
3. 在 `src/composables/` 添加业务逻辑
4. 在 `src/stores/` 添加状态管理（如需要）
5. 在 `src/components/` 添加 UI 组件
6. 在 `src/views/` 添加页面（如需要）

### 代码审查检查项

- [ ] 文件放置在正确的目录
- [ ] 命名符合规范
- [ ] 添加了必要的注释
- [ ] 类型定义完整
- [ ] 没有循环依赖
- [ ] 代码复用性良好
