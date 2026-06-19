# user_v2 开发路线图

## 已完成

| 模块 | 内容 | 状态 |
|------|------|------|
| **认证** | 登录/注册/登出/Token刷新 | ✅ |
| **WebSocket** | 连接/认证/心跳/重连/消息队列 | ✅ |
| **房间** | 房间列表/创建/加入/离开/成员管理 | ✅ |
| **私聊** | 创建私聊/列表/消息发送 | ✅ |
| **消息** | 消息列表/发送(乐观更新)/回复/编辑/删除 | ✅ |
| **好友** | 好友列表/请求收发/处理/删除 + FriendsView | ✅ |
| **发现** | 公开房间/搜索/用户搜索 + DiscoverView | ✅ |
| **侧边栏** | ChatRoomList + 私聊区 + 直接房间 | ✅ |
| **通知** | HTTP API/Store/通知面板(骨架) | ✅ |
| **设置** | ProfileView 偏好/外观/安全/通知/隐私等 | ✅ |
| **主题** | 亮/暗/跟随系统 + 个性化 + QuickBar | ✅ |
| **国际化** | zh/en/ja 三语言 | ✅ |
| **邀请** | API/Store/InviteValidationView | ✅ |
| **文件上传** | API/Store + ChatInputArea 文件发送 | ✅ |
| **路由** | 全部路由 + 导航守卫 + NavBar | ✅ |
| **Typing Indicator** | 输入指示器 + WS 事件订阅 + 5s 自动超时 | ✅ |
| **连接状态指示** | AppView banner — 断线/重连/连接中 | ✅ |
| **在线状态** | UserStatusChanged WS → room store → ChatMemberPanel/ChatHeader | ✅ |
| **离线消息拉取** | 重连后 GetMissedMessages → 去重插入消息列表 | ✅ |
| **消息搜索** | SearchMessagesPanel 弹窗 + API + 键盘导航 | ✅ |
| **房间管理** | RoomSettingsModal (General/Members/Invitations 三标签) | ✅ |
| **邀请链接 UI** | 生成/复制/撤销邀请码 | ✅ |
| **个人头像上传** | ProfileView 点击上传 + 预览 | ✅ |
| **浏览器推送通知** | Notification API + NewMessage 事件 → 系统通知 | ✅ |
| **消息反应** | 后端: DB迁移/Model/Service/API/WS + MessageResponse.reactions | ✅ |
| **已读回执** | 消息气泡显示已读/未读状态 + Store 自动发送 + 设置开关 | ✅ |

### 本次会话修复/补全

- [x] 修复 `useMessageActions` 中 roomId 过期导致消息发送失败
- [x] 修复 `selectRoom`/`selectDirectRoom` 缺少 `JoinRoom` WS 消息
- [x] 修复 http.ts 拦截器中 `/auth/login` 401 被吞
- [x] 修复 friend API 路径缺少 `/users` 前缀
- [x] 修复 `getMyRooms` 返回类型
- [x] 修复 settings API 路径/方法
- [x] 修复 `storeToRefs` → `computed` 渲染崩溃
- [x] FriendsView / DiscoverView 完整实现
- [x] 内联 SVG → Element Plus 图标统一替换
- [x] 修复 `friend.ts` TS undefined 非空断言
- [x] 修复 `typingUsers` 模板绑定 `.value`
- [x] 修复 `updateMemberStatus`、`UpdateRoomData` store export 遗漏
- [x] 移除 `useMessageActions.jumpToMessage` 死代码（实际跳转功能已通过 ChatMessageList.scrollToMessage 实现）
- [x] 增强图片懒加载重试机制：自动重试(最多2次) + 手动点击重试
- [x] 补全缺失的 git 跟踪文件：`BaseButton.vue`、`BaseBadge.vue`、`avatar.ts`

---

## 待完成（按优先级排列）

### Phase A — user_v2 功能补齐 ✅

| # | 任务 | 状态 |
|---|------|:----:|
| A1 | **消息反应 UI** — 气泡加表情按钮 → Emoji 选择器 → 显示反应列表 | ✅ |
| A2 | **编辑历史查看** — "已编辑" 可点击 → 弹出 EditHistoryPanel | ✅ |
| A3 | **Emoji 选择器** — ChatInputArea 加 Emoji 按钮 → 弹出面板 | ✅ |
| A4 | **Markdown 渲染** — 消息内容支持 Markdown 显示 | ✅ |
| A5 | **语言切换 (NavBar)** — NavBar 添加语言切换按钮（循环切换） | ✅ |
| A6 | **GIF 选择器** — Giphy API + GifPicker 组件 | ✅ |
| A7 | **系统日志订阅** — WS Logs 实时查看界面 (user_v2 用户无权限) | ❌ 已取消 |
| A8 | **删除账号** — ProfileView 危险区域 + 后端自服务 API | ✅ |

### Phase A+ — UI 与功能补齐 ✅ 已完成

| # | 任务 | 涉及文件 | 预估 | 状态 |
|---|------|---------|:----:|:----:|
| A9 | **新建房间 UI** — CreateRoomModal + QuickDial + NavBar 入口接入 | `NavBar.vue`, `components/quick/CreateRoomModal.vue` | 2h | ✅ |
| A10 | **创建房间路由修复** — `useRoom.ts`/`NavBar.vue`/`constants/routes.ts` 跳转修复 | `composables/useRoom.ts:37`, `NavBar.vue:121`, `constants/routes.ts` | 0.5h | ✅ |
| A11 | **PinnedMessagesPanel 接入 AppView** — ChatHeader 置顶入口按钮 + AppView 右侧面板 | `ChatHeader.vue`, `AppView.vue`, `components/chat/index.ts` | 2h | ✅ |
| A12 | **组件导出修复** — barrel export 补全 `PinnedMessagesPanel`、`EditHistoryPanel`、`GifPicker`、`EmojiPicker` | `components/chat/index.ts` | 0.5h | ✅ |
| A13 | **全局错误边界** — ErrorBoundary 组件捕获渲染异常 | `components/error/ErrorBoundary.vue`, `components/error/index.ts` | 1h | ✅ |
| A14 | **用户卡片/弹窗** — UserProfileModal 查看用户资料/私聊/加好友 | `components/user/UserProfileModal.vue`, `components/user/index.ts` | 2h | ✅ |

### Phase B — 动画与用户体验优化 ✅ 已完成

> 设计规范参考: `prototype/new_user_client/` — 暗色主题 `#7c5cfc` 紫色品牌色, 过渡 100-200ms ease, 响应式断点 900/860/840/640px

| # | 任务 | 涉及文件 | 预估 | 状态 |
|---|------|---------|:----:|:----:|
| B1 | **消息进出动画** — 新消息滑入，删除/编辑平滑过渡 | `ChatMessageList.vue`, `ChatMessageBubble.vue` | 3h | ✅ |
| B2 | **页面转场动画** — 路由切换 Transition 效果 | `AppView.vue`, `router` | 2h | ✅ |
| B3 | **发送按钮微交互** — loading/success/error 状态反馈 | `ChatInputArea.vue` | 1.5h | ✅ |
| B4 | **滚动体验** — 新消息自动滚动、滚动到顶部加载历史时保持位置、跳转未读 | `ChatMessageList.vue` | 3h | ✅ |
| B5 | **骨架屏/加载态** — 房间列表/消息列表/发现页 Skeleton 占位 | `ChatRoomList.vue`, `ChatMessageList.vue`, `DiscoverView.vue` | 3h | ✅ |
| B6 | **Toast/通知动效** — 通知出现/消失动画 | `useNotification.ts`, `AppView.vue` | 1h | ✅ |
| B7 | **响应式适配** — 移动端布局、侧边栏抽屉、触摸优化 | `AppView.vue`, `NavBar.vue`, `ChatRoomList.vue` | 4h | ✅ |
| B8 | **在线状态指示器** — 头像绿点脉冲动画 + 状态切换过渡 | `ChatHeader.vue`, `ChatMemberPanel.vue` | 1.5h | ✅ |
| B9 | **图片/文件懒加载** — 渐进加载、占位图、自动重试(最多2次)+手动重试 | `ChatMessageBubble.vue` | 2h | ✅ |
| B10 | **主题切换平滑过渡** — 亮/暗切换时颜色过渡动画 | `useTheme.ts`, `AppView.vue` | 1h | ✅ |

### Phase C — 后端功能补齐

| # | 任务 | 涉及文件 | 预估 | 状态 |
|---|------|---------|:----:|:----:|
| C1 | **删除账号自服务 API** — `DELETE /api/v1/users/me` | `src/handlers/user.rs` | 2h | ✅ (随 A8 完成) |
| C2 | **置顶消息** — DB迁移/Model/Service/API/WS/Test (后端) + API/Store/Types/Components/WS (前端) | 后端已完善 `/src/`; 前端 `client/user_v2/src/` | ✅ 前端✅ 后端✅ ChatMessageList渲染、ChatHeader入口、PinnedMessagesPanel、messageStore pin/unpin、jumpToMessage跳转 |
| C3 | **消息线程** — parent_id 字段 + API + WS + UI | ❌ 已取消 |
| C4 | **Admin 管理面板** — `client/admin/` 已有完整实现 | ❌ 已取消 |

---

## 路由与视图映射

```
/                 → LandingView     [public]
/login            → LoginView       [public]
/register         → RegisterView    [public]
/invite/:code     → InviteValidationView [public]
/app              → AppView         [auth]  ← 核心聊天界面
  ├ ChatRoomList (侧边栏: 群聊 + 私聊)
  ├ ChatHeader (房间头 + 搜索/置顶/成员/设置按钮)
  ├ ChatMessageList (消息列表 + 输入指示器 + TransitionGroup动画)
  │  └ ChatMessageBubble (单条消息 + Markdown/图片/文件/反应/已编辑)
  ├ ChatInputArea (输入区 + 文件上传/emoji/GIF/回复/编辑)
  ├ ChatMemberPanel (成员面板 + 在线状态)
  ├ PinnedMessagesPanel (置顶消息面板)
  ├ SearchMessagesPanel (消息搜索弹窗)
  └ RoomSettingsModal (房间设置弹窗)
/profile          → ProfileView     [auth]  ← 用户设置 + 头像上传
/discover         → DiscoverView    [auth]  ← 发现/搜索
/friends          → FriendsView     [auth]  ← 好友管理
```

## 数据流

```
HTTP API (REST) ──→ stores ──→ views
                        ↑
WebSocket (实时) ────────┘
```

- 初始数据加载：通过 HTTP API 填充 store
- 实时更新：WebSocket 事件直接更新 store
- 消息发送：乐观更新（先显示，WS 确认后替换真实 ID）

## CRITICAL 注意事项

1. **store 访问** — 始终用 `computed(() => store.prop)` 而非 `storeToRefs` 避免渲染时的 undefined 问题
2. **WebSocket JoinRoom** — 切换房间时必须发送 `JoinRoom` WS 消息，否则后端 `is_user_in_room` 检查不通过
3. **reply_to** — 非回复消息时设为 `null`，勿传 `""`，后端 `Option<Uuid>` 解析空字符串会失败
4. **store return** — 新增 store 方法后必须在 return 块中导出，否则 type-check 会报 missing property
5. **路由跳转** — 创建/加入房间后导航到 `/app`（非 `/room/:id`），`ROUTE_PATHS.CHAT` 须为 `/app`
