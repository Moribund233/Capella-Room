# Phase 1.1: 社交功能扩展开发规划

> 对应后端 0007-0010 功能实现，扩展 User 端社交能力
> 创建时间: 2026-05-07

---

## 概述

Phase 1.1 是在 Phase 1-8 基础功能完成后的扩展阶段，主要实现社交相关功能：
- **0007**: 搜索功能（房间搜索、用户搜索）
- **0008**: 私聊功能（1对1私信）
- **0009**: 好友功能（好友管理、请求处理）
- **0010**: 房间邀请机制（邀请码、链接分享）

---

## 开发时间线

```
Week 1                    Week 2
├─────────────────────────┼─────────────────────────┤
│ Phase 1.1-1             │ Phase 1.1-2             │
│ 搜索功能 (0007)         │ 私聊功能 (0008)         │
│ 2-3 天                  │ 2-3 天                  │
├─────────────────────────┼─────────────────────────┤
│ Phase 1.1-3             │ Phase 1.1-4             │
│ 好友功能 (0009)         │ 房间邀请 (0010)         │
│ 3-4 天                  │ 2-3 天                  │
└─────────────────────────┴─────────────────────────┘
```

---

## Phase 1.1-1: 搜索功能 (0007)

**工期**: 2-3 天  
**目标**: 实现房间和用户的搜索功能，支持发现公开房间

### 任务清单

- [x] **1.1.1.1 类型定义**
  - [x] `SearchResult` 接口
  - [x] `SearchParams` 接口
  - [x] `SearchType` 枚举 (room/user)

- [x] **1.1.1.2 API 封装**
  - [x] `searchRooms(keyword)` - 搜索房间
  - [x] `searchUsers(keyword)` - 搜索用户
  - [x] `getPublicRooms()` - 获取公开房间列表

- [x] **1.1.1.3 Search Store**
  - [x] 搜索状态管理
  - [x] 搜索结果缓存
  - [x] 搜索历史记录

- [x] **1.1.1.4 发现页面 (DiscoverView)**
  - [x] 页面布局设计
  - [x] 搜索栏组件集成
  - [x] 结果展示区域

- [x] **1.1.1.5 搜索组件**
  - [x] `UniversalSearchBar` - 通用搜索栏
  - [x] `RoomSearchResults` - 房间搜索结果
  - [x] `UserSearchResults` - 用户搜索结果

- [x] **1.1.1.6 用户卡片组件**
  - [x] `UserCard` - 用户信息卡片
  - [x] `UserProfileModal` - 用户资料弹窗
  - [x] 显示用户在线状态

- [x] **1.1.1.7 路由与导航**
  - [x] 添加 `/discover` 路由
  - [x] 侧边栏添加发现入口
  - [x] 移动端适配

### 交付物

| 文件 | 说明 |
|------|------|
| `src/views/DiscoverView.vue` | 发现页面 |
| `src/components/search/UniversalSearchBar.vue` | 通用搜索栏 |
| `src/components/search/SearchFilterTabs.vue` | 搜索类型切换 |
| `src/components/search/RoomSearchResults.vue` | 房间搜索结果 |
| `src/components/search/UserSearchResults.vue` | 用户搜索结果 |
| `src/components/user/UserCard.vue` | 用户卡片 |
| `src/components/user/UserProfileModal.vue` | 用户资料弹窗 |
| `src/stores/search.ts` | 搜索状态管理 |
| `src/api/search.ts` | 搜索 API |
| `src/types/search.ts` | 搜索类型定义 |

### 验收标准

- [x] 可以通过关键词搜索房间
- [x] 可以通过关键词搜索用户
- [x] 搜索结果正确展示
- [x] 点击用户显示资料弹窗
- [x] 发现页面在三种屏幕尺寸下正常显示

### 状态

**✅ 已完成** - 2026-05-07
- 类型检查通过
- Lint 检查通过
- 代码已合并到主分支

---

## Phase 1.1-2: 私聊功能 (0008)

**工期**: 2-3 天  
**目标**: 实现 1对1 私聊功能，支持创建和查看私聊房间

### 任务清单

- [x] **1.1.2.1 类型定义**
  - [x] `RoomType` 枚举扩展 (Group/Direct)
  - [x] `DirectRoom` 接口
  - [x] `CreateDirectRoomData` 接口

- [x] **1.1.2.2 API 封装**
  - [x] `createDirectRoom(userId)` - 创建私聊房间
  - [x] `getDirectRooms()` - 获取私聊列表

- [x] **1.1.2.3 DirectRoom Store**
  - [x] 私聊房间状态管理
  - [x] 与群聊房间分离存储

- [x] **1.1.2.4 房间列表改造**
  - [x] `RoomTypeTabs` - 群聊/私聊标签切换
  - [x] `DirectRoomCard` - 私聊房间卡片
  - [x] 改造 `RoomList` 支持类型切换

- [x] **1.1.2.5 私聊头部组件**
  - [x] `DirectChatHeader` - 显示对方信息
  - [x] 在线状态显示
  - [x] 对方资料入口

- [x] **1.1.2.6 发起私聊入口**
  - [x] `StartDirectChatModal` - 发起私聊弹窗
  - [x] 从用户资料发起私聊
  - [x] 从搜索结果发起私聊

- [x] **1.1.2.7 私聊会话适配**
  - [x] 复用 `ChatRoomView` 支持私聊模式
  - [x] 私聊房间显示 DirectChatHeader
  - [x] 私聊消息处理（WebSocket 集成）
  - [x] 群聊功能（详情/成员管理）对私聊隐藏

### 交付物

| 文件 | 说明 |
|------|------|
| `src/components/room/RoomTypeTabs.vue` | 房间类型标签 |
| `src/components/room/DirectRoomCard.vue` | 私聊房间卡片 |
| `src/components/chat/DirectChatHeader.vue` | 私聊头部 |
| `src/components/user/StartDirectChatModal.vue` | 发起私聊弹窗 |
| `src/stores/directRoom.ts` | 私聊状态管理 |
| `src/api/directRoom.ts` | 私聊 API |
| `src/types/room.ts` (扩展) | 房间类型定义 |

### 验收标准

- [x] 可以从用户资料发起私聊
- [x] 私聊房间正确显示在列表中
- [x] 私聊消息正常收发
- [x] 私聊头部显示对方信息
- [x] 已存在的私聊直接跳转，不重复创建

### 状态

**✅ 已完成** - 2026-05-07
- ChatRoomView 支持私聊/群聊双模式
- 类型检查通过

---

## Phase 1.1-3: 好友功能 (0009)

**工期**: 3-4 天  
**目标**: 实现完整的好友管理系统，包括请求、列表、删除

### 任务清单

- [x] **1.1.3.1 类型定义**
  - [x] `Friend` 接口
  - [x] `FriendRequest` 接口
  - [x] `FriendRequestStatus` 枚举
  - [x] `FriendRequestAction` 枚举

- [x] **1.1.3.2 API 封装**
  - [x] `getFriends()` - 获取好友列表
  - [x] `sendFriendRequest(data)` - 发送好友请求
  - [x] `getReceivedRequests()` - 获取收到的请求
  - [x] `getSentRequests()` - 获取发送的请求
  - [x] `handleFriendRequest(id, action)` - 处理请求
  - [x] `cancelFriendRequest(id)` - 取消请求
  - [x] `deleteFriend(id)` - 删除好友

- [x] **1.1.3.3 Friend Store**
  - [x] 好友列表管理
  - [x] 请求列表管理
  - [x] 未读请求计数

- [x] **1.1.3.4 好友页面 (FriendsView)**
  - [x] 页面布局设计
  - [x] 好友列表标签
  - [x] 好友请求标签

- [x] **1.1.3.5 好友列表组件**
  - [x] `FriendList` - 好友列表容器
  - [x] `FriendCard` - 好友卡片
  - [x] 在线状态显示

- [x] **1.1.3.6 好友请求组件**
  - [x] `FriendRequestList` - 请求列表容器
  - [x] `FriendRequestCard` - 请求卡片
  - [x] 收到的请求（接受/拒绝）
  - [x] 发出的请求（取消）

- [x] **1.1.3.7 添加好友功能**
  - [x] `AddFriendModal` - 添加好友弹窗
  - [x] 搜索用户
  - [x] 发送请求表单（附加消息）
  - [x] 发送成功提示

- [x] **1.1.3.8 好友操作菜单**
  - [x] `FriendContextMenu` - 右键菜单
  - [x] 发送私信
  - [x] 删除好友
  - [x] 查看资料

- [x] **1.1.3.9 导航集成**
  - [x] 添加 `/friends` 路由
  - [x] 侧边栏添加好友入口
  - [x] 未读请求角标
  - [x] 移动端导航集成

### 交付物

| 文件 | 说明 |
|------|------|
| `src/views/FriendsView.vue` | 好友页面 |
| `src/components/friend/FriendList.vue` | 好友列表 |
| `src/components/friend/FriendCard.vue` | 好友卡片 |
| `src/components/friend/FriendRequestList.vue` | 请求列表 |
| `src/components/friend/FriendRequestCard.vue` | 请求卡片 |
| `src/components/friend/AddFriendModal.vue` | 添加好友弹窗 |
| `src/components/friend/FriendContextMenu.vue` | 好友操作菜单 |
| `src/stores/friend.ts` | 好友状态管理 |
| `src/api/friend.ts` | 好友 API |
| `src/types/friend.ts` | 好友类型定义 |

### 验收标准

- [x] 可以发送好友请求
- [x] 可以接收/拒绝好友请求
- [x] 好友列表正确显示
- [x] 可以删除好友
- [x] 未读请求有角标提示（桌面 + 移动端）
- [x] 从好友列表可以发起私聊（通过资料弹窗）

### 状态

**✅ 已完成** - 2026-05-07
- 类型检查通过
- Lint 检查通过

---

## Phase 1.1-4: 房间邀请机制 (0010)

**工期**: 2-3 天  
**目标**: 实现私有房间的邀请码机制，支持链接分享

### 任务清单

- [x] **1.1.4.1 类型定义**
  - [x] `RoomInvitation` 接口
  - [x] `CreateInvitationData` 接口
  - [x] `JoinByInviteData` 接口

- [x] **1.1.4.2 API 封装**
  - [x] `createInvitation(roomId, data)` - 创建邀请
  - [x] `getRoomInvitations(roomId)` - 获取邀请列表
  - [x] `revokeInvitation(roomId, invitationId)` - 撤销邀请
  - [x] `joinByInviteCode(code)` - 通过邀请码加入
  - [x] `validateInviteCode(code)` - 验证邀请码

- [x] **1.1.4.3 Invitation Store**
  - [x] 邀请列表管理
  - [x] 当前邀请状态

- [x] **1.1.4.4 邀请管理组件**
  - [x] `RoomInvitationManager` - 邀请管理面板
  - [x] `InvitationCard` - 邀请码卡片
  - [x] 显示邀请码/过期时间/使用次数

- [x] **1.1.4.5 创建邀请弹窗**
  - [x] `CreateInvitationModal` - 创建邀请弹窗
  - [x] 过期时间设置（可选）
  - [x] 最大使用次数设置（可选）
  - [x] 生成邀请码

- [x] **1.1.4.6 邀请分享功能**
  - [x] `InviteLinkShareModal` - 分享弹窗
  - [x] 复制邀请码
  - [x] 复制邀请链接

- [x] **1.1.4.7 通过邀请加入**
  - [x] `JoinByInviteModal` - 输入邀请码弹窗
  - [x] 邀请码验证
  - [x] 确认加入

- [x] **1.1.4.8 邀请验证页面**
  - [x] `InviteValidationView` - 邀请验证页面
  - [x] 路由 `/invite/:code`
  - [x] 无需登录可访问
  - [x] 登录后自动加入

- [x] **1.1.4.9 成员管理集成**
  - [x] 改造 `RoomMemberManager` 增加邀请标签
  - [x] 侧边栏添加"通过邀请码加入"入口

### 交付物

| 文件 | 说明 |
|------|------|
| `src/views/InviteValidationView.vue` | 邀请验证页面 |
| `src/components/room/RoomInvitationManager.vue` | 邀请管理面板 |
| `src/components/room/InvitationCard.vue` | 邀请码卡片 |
| `src/components/room/CreateInvitationModal.vue` | 创建邀请弹窗 |
| `src/components/room/InviteLinkShareModal.vue` | 分享邀请弹窗 |
| `src/components/room/JoinByInviteModal.vue` | 通过邀请加入弹窗 |
| `src/stores/invitation.ts` | 邀请状态管理 |
| `src/api/invitation.ts` | 邀请 API |
| `src/types/invitation.ts` | 邀请类型定义 |

### 验收标准

- [x] 房主可以创建邀请码
  - [x] 可以设置过期时间
  - [x] 可以设置使用次数限制
- [x] 可以复制邀请链接分享
- [x] 用户可以通过邀请码加入房间
- [x] 邀请验证页面正常工作
- [x] 侧边栏有"通过邀请码加入"入口

### 状态

**✅ 已完成** - 2026-05-07
- 类型检查通过
- Lint 检查通过
- 路由 /invite/:code 支持未登录访问

---

## 组件依赖关系

```
Phase 1.1-1 (搜索)
├── UserCard
│   └── UserProfileModal
│       ├── AddFriendModal (1.1-3)
│       └── StartDirectChatModal (1.1-2)
└── RoomCard (复用现有)

Phase 1.1-2 (私聊)
├── RoomTypeTabs
├── DirectRoomCard
└── DirectChatHeader

Phase 1.1-3 (好友)
├── FriendCard
│   └── FriendContextMenu
│       └── StartDirectChatModal (复用 1.1-2)
├── FriendRequestCard
└── AddFriendModal
    └── UserCard (复用 1.1-1)

Phase 1.1-4 (邀请)
├── InvitationCard
├── CreateInvitationModal
├── InviteLinkShareModal
└── JoinByInviteModal
```

---

## 路由规划

```typescript
// constants/routes.ts 扩展

export const ROUTE_NAMES = {
  // ... 现有
  DISCOVER: 'discover',
  FRIENDS: 'friends',
  INVITE: 'invite',
} as const

export const ROUTE_PATHS = {
  // ... 现有
  DISCOVER: '/discover',
  FRIENDS: '/friends',
  INVITE: '/invite/:code',
} as const
```

---

## Store 架构

```
stores/
├── search.ts          # Phase 1.1-1
├── directRoom.ts      # Phase 1.1-2
├── friend.ts          # Phase 1.1-3
└── invitation.ts      # Phase 1.1-4
```

---

## 开发检查清单

### 每个 Phase 开始前

- [ ] 阅读后端 API 文档
- [ ] 确认设计规范
- [ ] 创建功能分支 `feature/phase-1.1-x`

### 每个 Phase 进行中

- [ ] 遵循现有代码规范
- [ ] 添加函数级注释
- [ ] 自测功能正常
- [ ] 类型检查通过 (`pnpm type-check`)

### 每个 Phase 结束后

- [ ] 代码审查
- [ ] 合并到主分支
- [ ] 更新进度文档
- [ ] 标记任务完成

---

## 风险预案

| 风险 | 应对措施 |
|------|----------|
| 好友/私聊逻辑复杂 | 优先实现基础功能，高级功能后续迭代 |
| 邀请码安全性 | 与后端确认验证逻辑，前端做好错误处理 |
| 搜索性能问题 | 添加防抖，限制最小搜索字符数 |
| 进度延迟 | 适当调整范围，保证核心功能可用 |

---

## 附录：开发顺序图

```
Phase 1.1-1: 搜索功能 (0007)
    │
    ├── 基础: 类型定义 + API + Store
    ├── 组件: 搜索栏 + 结果列表
    ├── 页面: DiscoverView
    └── 集成: 路由 + 导航
    │
    ▼
Phase 1.1-2: 私聊功能 (0008)
    │
    ├── 基础: 类型扩展 + API + Store
    ├── 组件: RoomTypeTabs + DirectRoomCard
    ├── 改造: RoomList 支持类型切换
    └── 入口: UserProfileModal 添加私聊按钮
    │
    ▼
Phase 1.1-3: 好友功能 (0009)
    │
    ├── 基础: 类型定义 + API + Store
    ├── 页面: FriendsView
    ├── 组件: FriendList + FriendRequestList
    ├── 组件: AddFriendModal
    └── 集成: 导航角标 + 通知
    │
    ▼
Phase 1.1-4: 房间邀请 (0010)
    │
    ├── 基础: 类型定义 + API + Store
    ├── 组件: RoomInvitationManager
    ├── 组件: 创建/分享/加入弹窗
    ├── 页面: InviteValidationView
    └── 集成: RoomMemberManager 标签
    │
    ▼
  完成
```

---

## 关联文档

- [后端功能文档](../../../docs/v1/backend-refinement.md) - 0007-0010 后端实现详情
- [架构设计](./architecture.md) - 前端架构规范
- [设计系统](./design-system.md) - UI/UX 设计规范
- [API 集成](./api-integration.md) - API 调用规范

---

*文档版本: 1.0*  
*最后更新: 2026-05-07*
