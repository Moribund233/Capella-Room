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

#### 网络层
- Retrofit + Moshi codegen + OkHttp 条件认证拦截器
- Hilt 注入 NetworkModule（单 Retrofit 实例）
- AuthApi、UserApi、RoomApi 接口定义
- 开发环境网络策略（`network_security_config.xml` → 允许 `10.0.2.2:3000` HTTP）

#### 认证
- TokenManager（DataStore 持久化 JWT）
- AuthRepository（login / register / refresh / logout）
- LoginViewModel 接入真实 API → 登录成功跳转频道列表

---

## 待开发

### Phase 4: 频道列表页面（参照 `channels.html`）
- 搜索栏
- 在线用户横滚列表
- 分类筛选标签（全部/频道/私信/未读）
- 频道/DM 列表 + 未读徽章
- FAB 创建房间
- Bottom Navigation（首页/消息/发现/我的）

### Phase 5: 核心聊天界面（参照 `chat.html`）
- 消息气泡 + 渐变色头像
- 表情反应（toggle 状态）
- 消息操作（回复/反应/更多）
- 内联回复预览
- 输入区域（表情面板 + 附件 + 自动调整大小输入框 + 发送）
- WebSocket 消息收发
- 消息历史游标分页加载

### Phase 6: 话题线程详情（参照 `thread.html`）
- 分屏-详细视图
- OP 标记
- 回复列表 + 排序
- "回复给" 指示器

### Phase 7: 个人资料 / 设置（参照 `profile.html`）
- 用户头像编辑
- 统计卡片（消息数/频道数/好友数）
- 偏好开关（深色模式、通知、声音）
- 账号设置链接（隐私、已连接账号）
- 危险区域（退出登录）

### Phase 8: WebSocket 客户端
- 连接管理（Auth/Reconnect 握手）
- 心跳（30s Ping/Pong）
- 消息收发（ChatMessage、Typing、MessageRead）
- 房间事件订阅（UserJoined、UserLeft、OnlineUsers）
- 断线重连 + 消息补发

### Phase 9: 构建发布
- ProGuard 混淆规则
- Release 签名配置
- 版本管理 CI

---

## 技术栈

| 层 | 选型 | 版本 |
|---|---|---|
| UI | Jetpack Compose + Material 3 | BOM 2024.02 |
| DI | Hilt | 2.50 |
| 导航 | Navigation Compose | 2.7.7 |
| 网络 | Retrofit + Moshi + OkHttp | 2.9.0 / 1.15.0 / 4.12.0 |
| 图片 | Coil (Compose) | 2.5.0 |
| 状态 | ViewModel + StateFlow | 2.7.0 |
| 存储 | DataStore Preferences | 1.0.0 |
| WebSocket | Java-WebSocket | 1.5.5 |
| 构建 | Gradle Kotlin DSL + Version Catalog | AGP 8.3.0 / Kotlin 1.9.23 |

## 后端

| 端点 | 说明 |
|---|---|
| `http://10.0.2.2:3000/api/v1/auth/*` | 认证（登录/注册/刷新） |
| `http://10.0.2.2:3000/api/v1/users/*` | 用户信息/设置 |
| `http://10.0.2.2:3000/api/v1/rooms/*` | 房间/频道 |
| `ws://10.0.2.2:3000/ws` | WebSocket 实时通信 |
