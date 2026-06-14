# Phase B: 动画与用户体验优化设计

## 背景

基于 `client/user_v2/docs/roadmap.md` Phase B 及 `prototype/new_user_client/` 设计体系，对 user_v2 前端进行动画与 UX 优化。

## 设计原则

- 保持现有动效时长（0.1s-0.25s），仅补全缺失动画
- 响应式对齐原型多断点系统：900px / 860px / 840px / 640px
- 颜色/主题 token 对齐原型暗色体系（紫色 `#7c5cfc` 品牌色）

---

## 分组一：动画组（B1/B2/B3/B6/B8/B10）

### B1 — 消息进出动画

**现状**: `ChatMessageList.vue` 已有 `TransitionGroup name="msg"`，定义了 `msg-enter-active`（0.25s ease-out）、`msg-leave-active`（0.2s ease-in）、`msg-move`（0.25s ease）。`ChatMessageBubble.vue` 有 `transition: background 0.1s`。

**缺失**:
- 删除消息时列表项没有平滑收起效果（现有 leave 动画设了 `position: absolute` 但缺少高度过渡）
- 新消息滑入偏移量略大（12px），可微调

**计划**:
- `ChatMessageList.vue`: 优化 TransitionGroup leave 动画，添加 `max-height` 过渡让删除项平滑收起
- `ChatMessageBubble.vue`: 消息行 hover 背景过渡已存在（0.1s），无需改动

### B2 — 页面转场动画

**现状**: `AppView.vue` 有 `banner-slide`、`fade`、`slide-left`、`slide-right` 四个 transition，但路由切换（`/app` ↔ `/profile` ↔ `/discover` ↔ `/friends`）**没有**过渡动画。

**计划**:
- `router/index.ts` 或 `App.vue`（根组件）添加 `<router-view>` 包裹 Transition
- 使用 `fade` 过渡（0.2s ease）—— 路由切换是页面级跳转，淡入淡出足够
- 不需要滑动过渡，因为路由不是同级面板切换

### B3 — 发送按钮微交互

**现状**: `ChatInputArea.vue` 发送按钮是 Element Plus `<el-button>`，点击后无 loading/成功/失败状态反馈。

**计划**:
- 发送中：按钮设为 `loading` 状态（`v-loading` 或 `:loading="isSending"`）
- 发送失败：按钮短暂变红色（0.3s 后恢复），替代现在无反馈
- 使用现有 `useMessageActions.ts` 的发送状态

### B6 — Toast/通知动效

**现状**: 当前使用 Element Plus 的 `ElNotification`/`ElMessage`，自带进场/离场动画。

**计划**:
- 确认现有 ElNotification 动画已满足需求（进场滑入 + 离场滑出），无需额外实现
- 如果现有调用未使用 ElNotification 的地方，统一改为 ElNotification

### B8 — 在线状态指示器

**现状**: `ChatHeader.vue` 已有绿点脉冲动画（`status-pulse` keyframes），`ChatMemberPanel.vue` 使用 Element Plus `<el-badge>` 或纯 CSS 点。

**计划**:
- 确认 `ChatMemberPanel.vue` 成员列表头像旁的在线状态点，如果缺失则添加
- 添加状态切换过渡（online ↔ offline 切换时 0.2s 渐变）

### B10 — 主题切换平滑过渡

**现状**: `useTheme.ts` 切换亮/暗主题时通过切换 CSS class 实现，颜色瞬间变化。

**计划**:
- 在 `:root` 或 `body` 级别添加 `transition: background-color 0.3s ease, color 0.3s ease`
- 对主要 CSS 自定义属性（`--bg`, `--surface`, `--fg`, `--border` 等）应用过渡

---

## 分组二：加载组（B5/B9）

### B5 — 骨架屏/加载态

**现状**: `ChatMessageList.vue` 已有简易骨架屏（`message-skeleton` + `pulse` 动画）。`ChatRoomList.vue` 和 `DiscoverView.vue` **没有**骨架屏。

**计划**:
- `ChatRoomList.vue`: 添加 Skeleton 组件，列表加载时显示占位行（头像 + 两行文字）
- `ChatMessageList.vue`: 现有骨架屏保留，不需要改
- `DiscoverView.vue`: 添加卡片式 Skeleton，加载时显示 4-6 个占位卡片

### B9 — 图片/文件懒加载

**现状**: `ChatMessageBubble.vue` 图片使用 `loading="lazy"` 属性，无占位图、无加载失败重试。

**计划**:
- 图片加载中：显示低分辨率纯色占位或 Element Plus `<el-skeleton>` 占位
- 图片加载失败：显示失败占位图标 + "点击重试"（移除 `style.display = 'none'`，改为显示占位）
- 文件附件：添加进度条（上传中）

---

## 分组三：布局组（B4/B7）

### B4 — 滚动体验

**现状**: `ChatMessageList.vue` 已有基础滚动功能：
- `autoScroll` 检测用户是否在底部
- 新消息自动滚到底部
- 滚动到顶部加载历史（`onScrollTop`）
- `scrollToMessage` 跳转到指定消息
- `newMessageCount` 未读计数 + 跳转按钮

**缺失**:
- "跳转到未读" 按钮在消息列表顶部（加载历史后快速回到第一条未读）
- 当前 `newMessageCount` 只计数新消息条数，缺少"跳到底部"浮动按钮

**计划**:
- 添加"跳到底部"浮动按钮（仅在未到底部时显示）
- 打开房间时自动滚动到底部

### B7 — 响应式适配

**现状**: 单断点 `640px`，移动端侧边栏作抽屉。缺少中间断点处理。

**计划**（对齐原型多断点）:

| 断点 | 行为 |
|------|------|
| `<= 900px` | 隐藏成员面板（`ChatMemberPanel`） |
| `<= 860px` | 隐藏右侧面板 |
| `<= 840px` | 网格布局（`grid-2/3/4`）折叠为单列 |
| `<= 640px` | 侧边栏变为 overlay 抽屉 + 隐藏搜索栏 + header 紧凑 |

**实现**:
- `useResponsive.ts`: 扩展当前 composable，添加多个断点响应式状态
- `AppView.vue`: 条件显示成员面板/右侧面板
- `NavBar.vue`: 移动端隐藏搜索栏

---

## 实现顺序

按分组并行执行：
1. 先处理**加载组（B5/B9）**——骨架屏和懒加载独立性强，风险低
2. 然后**布局组（B4/B7）**——滚动体验 + 响应式
3. 最后**动画组（B1/B2/B3/B6/B8/B10）**——细化动效
