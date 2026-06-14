# Phase B: 动画与用户体验优化 — 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Phase B 全部 10 项动画与用户体验优化（B1-B10）

**Architecture:** 分三组并行：加载组（B5/B9）→ 布局组（B4/B7）→ 动画组（B1/B2/B3/B6/B8/B10）。每项改动范围受限，涉及 `components/chat/`、`views/`、`composables/`。

**Tech Stack:** Vue 3 + TypeScript + SCSS + Element Plus

---

## 文件清单

| 文件 | 操作 | 分组 |
|------|------|------|
| `src/components/chat/ChatRoomList.vue` | 修改 | 加载组 |
| `src/views/DiscoverView.vue` | 修改 | 加载组 |
| `src/components/chat/ChatMessageBubble.vue` | 修改 | 加载组 |
| `src/components/chat/ChatMessageList.vue` | 修改 | 布局组 |
| `src/composables/useResponsive.ts` | 修改 | 布局组 |
| `src/views/AppView.vue` | 修改 | 布局组 |
| `src/App.vue` | 修改 | 动画组 |
| `src/components/chat/ChatInputArea.vue` | 修改 | 动画组 |
| `src/components/chat/ChatMemberPanel.vue` | 修改 | 动画组 |

---

## 分组一：加载组（B5 + B9）

### Task 1: ChatRoomList 骨架屏增强

**Files:**
- Modify: `src/components/chat/ChatRoomList.vue`

- [ ] **Step 1: 替换骨架屏为 el-skeleton**

将现有 `.channel-skeleton` 手工 CSS 替换为 Element Plus `<el-skeleton>` 组件，隐藏 `#` hash 符号，只保留名称占位。

修改 `ChatRoomList.vue` template 中的加载块（搜索现有 `<div v-if="roomStore.loading" class="channel-loading">`）:
```html
<div v-if="roomStore.loading" class="channel-loading">
  <div v-for="i in 5" :key="i" class="channel-skeleton">
    <el-skeleton :rows="1" animated />
  </div>
</div>
```

- [ ] **Step 2: 清理旧 CSS**

删除 `.channel-skeleton`、`.skeleton-hash`、`.skeleton-name`、`@keyframes pulse`（如无其他引用）。

保留 `.channel-loading` padding。

- [ ] **Step 3: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 2: ChatMessageBubble 图片懒加载增强

**Files:**
- Modify: `src/components/chat/ChatMessageBubble.vue`

- [ ] **Step 1: 添加图片加载状态变量**

在 `<script setup>` 中添加：
```ts
const imageLoaded = ref(false)
const imageError = ref(false)
```

- [ ] **Step 2: 替换图片模板**

将现有 `<img>`（搜索 `v-else-if="isImageUrl"`）替换为：
```html
<div v-else-if="isImageUrl" class="bubble-image">
  <div v-if="!imageLoaded && !imageError" class="bubble-image__placeholder">
    <el-skeleton animated style="width: 100%; height: 200px" />
  </div>
  <img
    v-show="imageLoaded && !imageError"
    :src="message.content"
    alt=""
    loading="lazy"
    @click.stop
    @load="imageLoaded = true"
    @error="imageError = true"
  />
  <div v-if="imageError" class="bubble-image__error" @click="imageError = false; imageLoaded = false; $nextTick(() => { imageLoaded = false })">
    <el-icon :size="24"><WarningFilled /></el-icon>
    <span>加载失败，点击重试</span>
  </div>
</div>
```

- [ ] **Step 3: 添加加载状态 CSS**

在 style 中添加：
```scss
.bubble-image {
  min-height: 60px;
  position: relative;

  &__placeholder {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  &__error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 32px;
    color: var(--muted);
    font-size: 13px;
    cursor: pointer;
    min-height: 100px;

    &:hover {
      color: var(--accent);
    }
  }
}
```

- [ ] **Step 4: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

## 分组二：布局组（B4 + B7）

### Task 3: 扩展 useResponsive 多断点

**Files:**
- Modify: `src/composables/useResponsive.ts`

- [ ] **Step 1: 添加原型对齐断点**

在 `breakpoints` 对象中补充原型对齐值：
```ts
const breakpoints = {
  xs: 0,
  s: 640,
  m: 768,
  l: 1024,
  xl: 1280,
  xxl: 1536,

  // 原型设计断点
  memberPanel: 900,
  rightPanel: 860,
  gridCollapse: 840,
  mobileSidebar: 640,
}
```

- [ ] **Step 2: 添加新 computed**

在 return 前添加：
```ts
const showMemberPanel = computed(() => windowWidth.value > 900)
const showRightPanel = computed(() => windowWidth.value > 860)
const isGridCollapsed = computed(() => windowWidth.value < 840)
```

- [ ] **Step 3: 导出新属性**

在 return 中添加 `showMemberPanel`, `showRightPanel`, `isGridCollapsed`。

- [ ] **Step 4: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 4: 响应式适配 AppView

**Files:**
- Modify: `src/views/AppView.vue`

- [ ] **Step 1: 导入新增响应式属性**

将 `const { isMobile } = useResponsive()` 改为：
```ts
const { isMobile, showMemberPanel } = useResponsive()
```

- [ ] **Step 2: 条件渲染成员面板**

将现有成员面板条件 `<div v-if="hasRoom && showMemberPanel && !isMobile"` 改为：
```html
<div
  v-if="hasRoom && showMemberPanel && showMemberPanel"
  class="app-view__member-panel"
>
```

- [ ] **Step 3: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 5: 滚动体验增强（跳到底部按钮）

**Files:**
- Modify: `src/components/chat/ChatMessageList.vue`

- [ ] **Step 1: 添加 scrollToBottom 按钮**

在模板末尾（`</div>` 容器关闭前），新消息提示之后添加：
```html
<transition name="fade">
  <div
    v-if="!autoScroll && messages.length > 0"
    class="scroll-bottom-btn"
    @click="scrollToNewMessages"
  >
    <el-icon><ArrowDown /></el-icon>
  </div>
</transition>
```

- [ ] **Step 2: 添加浮动按钮 CSS**

```scss
.scroll-bottom-btn {
  position: absolute;
  bottom: 16px;
  right: 24px;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  z-index: 10;
  transition: transform 0.15s ease, opacity 0.15s ease;

  &:hover {
    transform: scale(1.1);
  }

  &:active {
    transform: scale(0.95);
  }
}
```

注意: `.messages-container` 需要 `position: relative` 让绝对定位生效。

- [ ] **Step 3: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

## 分组三：动画组（B1 + B2 + B3 + B6 + B8 + B10）

### Task 6: B1 — 消息离场动画优化

**Files:**
- Modify: `src/components/chat/ChatMessageList.vue`

- [ ] **Step 1: 优化 leave 动画**

在现有 `msg-leave-active` 中添加 `max-height` 和 `margin-bottom` 过渡：
```scss
.msg-leave-active {
  transition: all 0.25s ease-in;
  max-height: 200px;
  margin-bottom: 0;
  overflow: hidden;
}
.msg-leave-to {
  opacity: 0;
  transform: translateX(-20px);
  max-height: 0;
  margin-bottom: 0;
  padding-top: 0;
  padding-bottom: 0;
}
```

将现有 `.msg-leave-to` 合并进上面的定义，删除旧 `.msg-leave-to`。

- [ ] **Step 2: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 7: B2 — 页面转场动画

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 包裹 `<router-view>` 添加 Transition**

```html
<template>
  <router-view v-slot="{ Component }">
    <transition name="page-fade" mode="out-in">
      <component :is="Component" />
    </transition>
  </router-view>
</template>
```

- [ ] **Step 2: 添加 Transition CSS**

在 `<style>` 中添加：
```scss
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.2s ease;
}

.page-fade-enter-from,
.page-fade-leave-to {
  opacity: 0;
}
```

- [ ] **Step 3: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 8: B3 — 发送按钮微交互

**Files:**
- Modify: `src/components/chat/ChatInputArea.vue`

- [ ] **Step 1: 添加发送状态 ref**

在 `<script setup>` 中添加：
```ts
const sendState = ref<'idle' | 'sending' | 'error'>('idle')
let sendStateTimer: ReturnType<typeof setTimeout> | null = null

function handleSend() {
  if (!inputText.value.trim() || sendState.value === 'sending') return
  sendState.value = 'sending'
  emit('send', inputText.value.trim())
  inputText.value = ''
  autoResize()

  // 监听发送结果（通过 message store 的状态）
  // 由于是乐观更新，发送后快速回到 idle
  setTimeout(() => {
    sendState.value = 'idle'
  }, 300)
}
```

- [ ] **Step 2: 按钮绑定状态**

将现有发送按钮改为：
```html
<button
  :title="t('chat.send')"
  class="send-btn"
  :class="{
    'send-btn--active': inputText.trim(),
    'send-btn--sending': sendState === 'sending',
    'send-btn--error': sendState === 'error',
  }"
  :disabled="!inputText.trim() || sendState === 'sending'"
  @click="handleSend"
>
  <el-icon v-if="sendState === 'sending'" class="is-loading" :size="20"><Loading /></el-icon>
  <el-icon v-else :size="20"><Promotion /></el-icon>
</button>
```

- [ ] **Step 3: 添加 CSS**

```scss
.send-btn--sending {
  opacity: 0.7;
  pointer-events: none;
}

.send-btn--error {
  color: var(--accent-pink) !important;
}
```

- [ ] **Step 4: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 9: B8 — 在线状态切换过渡

**Files:**
- Modify: `src/components/chat/ChatMemberPanel.vue`

- [ ] **Step 1: 添加状态点过渡**

在 `.member-dot` 中添加：
```scss
.member-dot {
  transition: background 0.2s ease, box-shadow 0.2s ease;
}
```

- [ ] **Step 2: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 10: B10 — 主题切换平滑过渡

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: 在全局样式中添加主题过渡**

在 `<style>` 中添加：
```scss
:root {
  transition: background-color 0.3s ease, color 0.3s ease;
}

* {
  transition: background-color 0.3s ease, border-color 0.3s ease, color 0.3s ease;
}
```

- [ ] **Step 2: lint + type-check**

```bash
npm run lint && npm run type-check
```

---

### Task 11: B6 — Toast/通知动效确认

**Files:** 无

- [ ] **Step 1: 检查通知调用**

搜索 `ElNotification`、`ElMessage`、`ElMessageBox` 调用，确认均已使用 Element Plus 内置动画。不需要改动。如代码中有 `console.log`/`alert` 等替代通知的地方，需提 issue 但不在本轮 Phase B 范围。

```bash
rg "ElNotification|ElMessage|\.message\(" src/ --include "*.ts" --include "*.vue"
```

确认现有调用已使用 Element Plus 组件后，标记完成。

---

## 验证

每项任务完成后运行：
```bash
npm run type-check
npm run lint
```

全部完成后：
```bash
npm run build-only
```
