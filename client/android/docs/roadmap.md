# Capella Room Android Client — 开发路线图

> 基于 Jetpack Compose + Material 3 + Hilt 构建，参照 `prototype/android_app/` HTML 原型逐步实现。

---

## 已完成

### Phase 1: 项目初始化 & 启动页 ✅

**2026-06-03**

- 移除旧 XML View 骨架代码（LoginActivity、MainActivity、layout XML）
- 搭建 Compose 项目结构（Version Catalog、Hilt DI、Navigation Compose）
- 创建品牌主题系统（`Color.kt`、`Type.kt`、`Theme.kt`）
- 实现 SplashScreen（2.2s Logo + pulse 呼吸动画 → 淡出）
- 实现 Landing 首页（Hero + 聊天预览卡片 + 功能特性网格 + CTA 按钮）
- 聊天气泡交错入场动画
- 转换 favicon 为 VectorDrawable 图标（启动器 + 应用内一致）

### Phase 2: 登录 / 注册页面 ✅

**2026-06-03**

- 双 Tab 切换（登录 / 注册）带 AnimatedContent 过渡
- 邮箱/密码表单 + 密码可见性切换 + 表单验证
- "记住我" + "忘记密码" 选项
- Google / GitHub 品牌图标登录按钮
- 底部服务条款链接
- 错误信息提示条

### Phase 3: 数据层 & 网络层 ✅

**2026-06-03**

#### 数据模型
- 统一响应封装 `ApiResponse<T>`、`PaginatedData<T>`
- Auth DTO（LoginRequest/Response, RegisterRequest, TokenData, UserDto）
- Room DTO（RoomDto, MessageDto, MessageListDto, CreateRoomRequest）
- 补充 DTO：DirectRoomDto, RoomMemberDto, UpdateRoleRequest, Invitation 系列

#### 网络层
- Retrofit + Moshi codegen + OkHttp 条件认证拦截器
- Hilt 注入 NetworkModule（单 Retrofit 实例）
- AuthApi、UserApi、RoomApi 接口定义
- 开发环境网络策略（`network_security_config.xml` → 允许 `10.0.2.2:3000` HTTP）

#### 认证
- TokenManager（DataStore 持久化 JWT）
- AuthRepository（login / register / refresh / logout）
- LoginViewModel 接入真实 API → 登录成功跳转频道列表
- 自动登录（SplashViewModel 检查 TokenManager 判断是否已登录）

### Phase 4: 频道列表页面（参照 `channels.html`）✅

**2026-06-04**

- 搜索栏（BasicTextField + placeholder）
- 在线用户横滚列表（渐变色头像 + 状态指示器）
- 分类筛选标签（全部/频道/私信/未读）
- 频道/DM 列表 + 未读徽章
- FAB 创建房间
- Bottom Navigation 组件（`CapellaBottomBar` — 首页/消息/发现/我的）
- MainScreen 容器管理底部导航切换
- 接入真实 API `GET /api/v1/users/me/rooms` + `GET /api/v1/rooms/direct/list`
- API 不可用时自动降级到本地缓存数据
- 后端 `get_my_rooms` 响应格式修复（裸数组 → `ApiResponse` 包裹）

### Phase 5: 核心聊天界面（参照 `chat.html`）✅

**2026-06-04**

- Header（返回按钮 + 频道名称 + 成员数 + 搜索/成员图标 + 连接状态指示器）
- 消息列表（LazyColumn reverseLayout，最新消息在底部）
- 日期分隔线（今天/昨天/M月d日）
- 消息气泡样式：
  - 当前用户：紫色气泡（#5865F2），右对齐，头像在右侧
  - 其他用户：深灰色气泡（#404249），左对齐，头像在左侧
  - 圆角设计区分发送者和接收者
- @提及紫色高亮
- 回到底部按钮（向上滑动超过2条消息时显示）
- 游标分页加载历史消息（cursor-based `before` 参数）
- 加载更多消息时保持滚动位置，不自动回到底部
- 输入区域：
  - 表情面板（20 个常用 emoji 横滚列表）
  - 附件按钮
  - 自动调整大小输入框（maxLines=5, maxHeight=120dp）
  - 发送按钮（有内容时激活）
- **WebSocket 实时消息收发**（通过 `WebSocketClient` + `WebSocketRepository`）
- 输入状态实时同步（"xxx 正在输入..."）
- 在线用户列表更新

### Phase 5.5: 发现页 ✅

**2026-06-04**

- 搜索公开频道
- 公开频道卡片（名称/描述/成员数/创建者）
- "加入" 按钮 → 调用 `POST /api/v1/rooms/:id/join`
- 点击卡片直接进入聊天

### Phase 5.6: 本地数据层 & 离线模式 ✅

**2026-06-04**

#### Room 数据库架构
- 数据库入口 `CapellaDatabase`（SQLite）
- 实体定义：
  - `MessageEntity` - 消息（含同步状态字段）
  - `RoomEntity` - 房间信息
  - `RoomMemberEntity` - 房间成员
  - `UserEntity` - 用户信息
  - `CurrentUserEntity` - 当前登录用户

#### DAO 接口
- `MessageDao` - 消息 CRUD、分页查询、同步状态查询
- `RoomDao` - 房间列表、未读计数、最后消息
- `UserDao` - 用户查询、当前用户管理

#### Repository 层
- `LocalMessageRepository` - 消息本地存储、同步状态管理
- `LocalRoomRepository` - 房间本地缓存
- `LocalUserRepository` - 用户本地存储

#### 离线模式特性
- 消息本地缓存，支持离线浏览历史消息
- 离线消息发送：保存为 PENDING 状态，网络恢复后自动同步
- 同步状态追踪：`SYNCED`, `PENDING`, `SENDING`, `FAILED`, `EDIT_PENDING`, `DELETE_PENDING`
- 房间列表本地缓存，API 失败时自动使用本地数据
- 登录状态本地持久化

#### 依赖注入
- `DatabaseModule` - Hilt 提供数据库、DAO、Repository 注入

### 数据层同步（对照后端 API v1 文档）✅

**2026-06-04**

- RoomApi 新增 12 个接口：DM 房间、邀请系统、成员管理、消息搜索/编辑/删除/历史
- UserApi 新增 17 个接口：用户搜索、密码修改、用户设置、好友系统、账号安全
- 修复后端 `get_my_rooms` 响应格式不一致
- 所有 API 对接完成，可正常收发数据

---

## 待开发

### Phase 6: 消息功能增强（高优先级）
- 消息编辑和删除（本地 + 远程同步）
- 消息搜索（本地缓存 + 远程 API）
- 消息反应（Emoji 表情回应）
- 文件/图片上传和预览
- 消息引用回复

### Phase 7: 话题线程详情（参照 `thread.html`）
- 分屏-详细视图
- OP 标记
- 回复列表 + 排序
- "回复给" 指示器

### Phase 8: 个人资料 / 设置（参照 `profile.html`）
- 用户头像编辑
- 统计卡片（消息数/频道数/好友数）
- 偏好开关（深色模式、通知、声音）
- 账号设置链接（隐私、已连接账号）
- 危险区域（退出登录）

### Phase 9: 好友系统
- 好友列表页面
- 添加/删除好友
- 好友在线状态
- 私信快速发起

### Phase 10: 通知系统
- 本地通知（新消息提醒）
- 通知设置（免打扰、关键词过滤）
- 未读消息角标

### Phase 11: 构建发布
- ProGuard 混淆规则
- Release 签名配置
- 版本管理 CI
- Google Play 上架准备



---

## 已实现的核心功能总结

### WebSocket 实时通信架构 ✅

**组件层次：**
```
ChatViewModel
    ├── WebSocketRepository (业务层)
    │       └── WebSocketClient (底层客户端)
    └── LocalMessageRepository (本地数据层)
```

**WebSocketClient 功能：**
- 基于 Java-WebSocket 库实现
- 自动连接和认证（JWT Token）
- 心跳检测（30s Ping/Pong）
- 断线重连（指数退避，最多10次）
- 支持重连恢复会话状态

**WebSocketRepository 功能：**
- 房间管理（join/leave）
- 消息收发（send/receive）
- 输入状态同步（typing/stop typing）
- 消息操作（edit/delete/mark read）
- 事件流分发（新消息、用户进出、在线列表等）

**消息协议支持：**
- `Auth` / `AuthResult` - 连接认证
- `ChatMessage` / `NewMessage` - 消息收发
- `Typing` / `UserTyping` / `StopTyping` - 输入状态
- `JoinRoom` / `RoomJoined` / `UserJoined` / `UserLeft` - 房间管理
- `OnlineUsers` - 在线用户列表
- `EditMessage` / `MessageEdited` - 消息编辑
- `DeleteMessage` / `MessageDeleted` - 消息删除
- `Ping` / `Pong` - 心跳

### 离线优先架构 ✅

**数据流向：**
```
发送消息: 用户输入 → 本地保存(PENDING) → WebSocket发送 → 服务器确认 → 更新状态(SYNCED)
接收消息: WebSocket接收 → 本地保存 → UI更新
离线消息: 本地保存(PENDING) → 网络恢复 → 自动同步
```

**同步状态：**
- `SYNCED` - 已同步
- `PENDING` - 待发送
- `SENDING` - 发送中
- `FAILED` - 发送失败（可重试）
- `EDIT_PENDING` - 编辑待同步
- `DELETE_PENDING` - 删除待同步

---

## 技术栈

| 层 | 选型 | 版本 |
|---|---|---|
| UI | Jetpack Compose + Material 3 | BOM 2024.02 |
| DI | Hilt | 2.50 |
| 导航 | Navigation Compose | 2.7.7 |
| 网络 | Retrofit + Moshi + OkHttp | 2.9.0 / 1.15.0 / 4.12.0 |
| 本地存储 | Room | 2.6.1 |
| 图片 | Coil (Compose) | 2.5.0 |
| 状态 | ViewModel + mutableStateOf | 2.7.0 |
| 偏好存储 | DataStore Preferences | 1.0.0 |
| WebSocket | Java-WebSocket | 1.5.5 |
| 构建 | Gradle Kotlin DSL + Version Catalog | AGP 8.3.0 / Kotlin 1.9.23 |

## 后端 API

| 端点 | 说明 | Android 集成状态 |
|---|---|---|
| `POST /api/v1/auth/*` | 认证（登录/注册/刷新） | ✅ 完成 |
| `POST /api/v1/users/logout` | 登出 | ✅ 完成 |
| `GET /api/v1/users/me` | 当前用户信息 | ✅ 完成 |
| `GET /api/v1/users/me/rooms` | 我的聊天室 | ✅ 频道列表 |
| `GET /api/v1/users` | 用户搜索 | ✅ 接口定义（待 UI） |
| `GET/PUT /api/v1/users/me/settings` | 用户设置 | ✅ 接口定义（待 UI） |
| `GET/POST /api/v1/rooms` | 房间 CRUD | ✅ 完成 |
| `GET /api/v1/rooms/:id/messages` | 消息历史（游标分页） | ✅ 聊天页 |
| `POST /api/v1/rooms/direct` | 创建/获取私聊 | ✅ 接口定义 |
| `POST /api/v1/rooms/:id/join` | 加入房间 | ✅ 发现页 |
| `GET/POST/DELETE /api/v1/rooms/:id/invitations` | 邀请管理 | ✅ 接口定义 |
| `PUT/DELETE /api/v1/messages/:id` | 消息编辑/删除 | ✅ 接口定义 |
| `GET /api/v1/messages/search` | 消息搜索 | ✅ 接口定义 |
| `POST /api/v1/upload/*` | 文件上传 | ✅ 接口定义 |
| `ws://10.0.2.2:3000/ws` | WebSocket 实时通信 | ✅ 完成（Phase 5）|

## 本地数据库 Schema

| 表名 | 用途 | 关键字段 |
|---|---|---|
| `messages` | 消息本地缓存 | id, room_id, content, sync_status, local_created_at |
| `rooms` | 房间信息缓存 | id, name, type, unread_count, last_message_at |
| `room_members` | 房间成员 | room_id, user_id, role, joined_at |
| `users` | 用户信息 | id, username, avatar_url, status |
| `current_user` | 当前登录用户 | user_id, username, access_token, refresh_token |
