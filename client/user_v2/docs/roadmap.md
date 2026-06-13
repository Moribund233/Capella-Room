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

---

## 待完成（按优先级排列）

### P1 — 功能补全

| # | 任务 | 涉及文件 | 后端就绪 | 预估 |
|---|------|---------|:-------:|:----:|
| 10 | **已读回执** — 消息气泡显示已读/未读状态 | `ChatMessageBubble.vue`, `message.ts` store | ⚠️ 部分 | 2h |

### P2 — 体验增强

| # | 任务 | 涉及文件 | 后端就绪 | 预估 |
|---|------|---------|:-------:|:----:|
| 12 | **Emoji 选择器** — ChatInputArea 加 Emoji 按钮 → 弹出 Emoji 面板 | `ChatInputArea.vue`, `EmojiPicker.vue` | N/A | 3h |
| 13 | **Markdown 渲染** — 消息内容支持 Markdown 格式显示 | `ChatMessageBubble.vue` | N/A | 2h |
| 14 | **编辑历史查看** — 已编辑消息显示 "已编辑"，可查看历史版本 | `ChatMessageBubble.vue`, `message.ts` API | ✅ | 2h |
| 15 | **语言切换 (已登录)** — NavBar 加语言切换 | `NavBar.vue`, `locale.ts` | N/A | 1h |
| 16 | **消息反应 (Reactions)** — 消息气泡加表情反应功能 | `ChatMessageBubble.vue`, types | ✅ 已完成 | 4h |
| 17 | **删除账号** — ProfileView 危险区域功能实现 | `ProfileView.vue`, `user.ts` API | ❌ 需确认 | 2h |

### P3 — 低优先级 / 锦上添花

| # | 任务 | 涉及文件 | 后端就绪 | 预估 |
|---|------|---------|:-------:|:----:|
| 18 | **GIF 选择器** | `ChatInputArea.vue` | N/A | 3h |
| 19 | **聊天背景自定义** | `ChatMessageList.vue`, personalization store | N/A | 2h |
| 20 | **置顶消息 (Pinned Message)** | new store + UI | ❌ 后端无 | 4h |
| 21 | **消息线程 (Threads)** | new store + UI | ❌ 后端无 | 8h |
| 22 | **系统日志订阅 (WS Logs)** | websocket types + UI | ✅ | 3h |

### Admin — 独立后台

| # | 任务 | 涉及文件 | 后端就绪 | 预估 |
|---|------|---------|:-------:|:----:|
| A1 | Admin 管理面板（独立前端或子模块） | `client/admin/` | ✅ | 长期 |

---

## 路由与视图映射

```
/                 → LandingView     [public]
/login            → LoginView       [public]
/register         → RegisterView    [public]
/invite/:code     → InviteValidationView [public]
/app              → AppView         [auth]  ← 核心聊天界面
  ├ ChatRoomList (侧边栏: 群聊 + 私聊)
  ├ ChatHeader (房间头 + 搜索/成员/设置按钮)
  ├ ChatMessageList (消息列表 + 输入指示器)
  │  └ ChatMessageBubble (单条消息 + 图片/文件渲染)
  ├ ChatInputArea (输入区 + 文件上传/emoji/GIF)
  └ ChatMemberPanel (成员面板 + 在线状态)
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
