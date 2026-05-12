# WebSocket 系统日志流

本文档描述 WebSocket 系统日志流的订阅机制和消息格式。

---

## 概述

系统日志流允许管理员通过 WebSocket 实时订阅服务端日志，便于实时监控和排查问题。

### 功能特性

- **实时推送**：服务端日志实时推送到客户端
- **级别过滤**：支持按日志级别过滤（error/warn/info/debug/all）
- **模块过滤**：支持按模块过滤（websocket/room/message/performance/all）
- **权限控制**：仅管理员可订阅系统日志

---

## 消息类型

### 客户端发送的消息

| 消息类型 | 说明 |
|----------|------|
| `SubscribeLogs` | 订阅系统日志 |
| `UnsubscribeLogs` | 取消订阅系统日志 |

### 服务端发送的消息

| 消息类型 | 说明 |
|----------|------|
| `LogEntry` | 系统日志条目（实时推送） |
| `LogSubscriptionConfirmed` | 日志订阅确认 |

---

## 订阅系统日志

管理员可以发送 `SubscribeLogs` 消息订阅系统日志：

```json
{
  "type": "SubscribeLogs",
  "payload": {
    "level": "info",
    "module": "websocket"
  }
}
```

**字段说明**:

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| level | string | 否 | 日志级别过滤：error/warn/info/debug/all，默认 all |
| module | string | 否 | 模块过滤：websocket/room/message/performance/all，默认 all |

### 日志级别

| 级别 | 说明 |
|------|------|
| `error` | 错误日志 |
| `warn` | 警告日志 |
| `info` | 信息日志 |
| `debug` | 调试日志 |
| `all` | 所有级别（默认） |

### 模块类型

| 模块 | 说明 |
|------|------|
| `websocket` | WebSocket 连接相关日志 |
| `room` | 房间管理相关日志 |
| `message` | 消息处理相关日志 |
| `performance` | 性能指标相关日志 |
| `all` | 所有模块（默认） |

---

## 订阅确认

订阅成功后，服务端会发送 `LogSubscriptionConfirmed`：

```json
{
  "type": "LogSubscriptionConfirmed",
  "payload": {
    "success": true,
    "message": "已订阅系统日志 (level: info, module: websocket)"
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| success | boolean | 订阅是否成功 |
| message | string | 确认消息 |

### 权限不足时的响应

如果非管理员用户尝试订阅，会收到错误：

```json
{
  "type": "LogSubscriptionConfirmed",
  "payload": {
    "success": false,
    "message": "权限不足，仅管理员可订阅系统日志"
  }
}
```

---

## 取消订阅

管理员可以发送 `UnsubscribeLogs` 取消订阅：

```json
{
  "type": "UnsubscribeLogs"
}
```

取消订阅后，服务端会停止推送日志消息。

---

## 实时日志推送

订阅成功后，服务端会实时推送 `LogEntry` 消息：

```json
{
  "type": "LogEntry",
  "payload": {
    "level": "info",
    "target": "capella_room::websocket::handler",
    "message": "User joined room: 550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2026-04-26T10:30:00.000Z",
    "fields": {
      "user_id": "44777268-d040-4ef5-81de-9aad6ea3ead3",
      "room_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| level | string | 日志级别：error/warn/info/debug |
| target | string | 日志模块/目标 |
| message | string | 日志消息内容 |
| timestamp | string (ISO 8601) | 日志时间戳 |
| fields | object \| null | 结构化日志字段（可选） |

---

## 前端处理建议

### 订阅日志

```javascript
// 示例：订阅 WebSocket 模块的 info 级别日志
function subscribeLogs(level = 'info', module = 'websocket') {
  socket.send(JSON.stringify({
    type: 'SubscribeLogs',
    payload: { level, module }
  }));
}

// 订阅所有日志
subscribeLogs('all', 'all');
```

### 接收日志

```javascript
// 示例：实时显示日志
socket.on('LogEntry', (log) => {
  const logElement = document.createElement('div');
  logElement.className = `log-entry log-${log.level}`;
  logElement.innerHTML = `
    <span class="log-time">${formatTime(log.timestamp)}</span>
    <span class="log-level">[${log.level.toUpperCase()}]</span>
    <span class="log-target">${log.target}</span>
    <span class="log-message">${log.message}</span>
  `;
  
  if (log.fields) {
    logElement.innerHTML += `<pre class="log-fields">${JSON.stringify(log.fields, null, 2)}</pre>`;
  }
  
  logContainer.appendChild(logElement);
  logContainer.scrollTop = logContainer.scrollHeight;
});
```

### 日志级别样式

```css
.log-error { color: #ff4d4f; background: #fff1f0; }
.log-warn { color: #faad14; background: #fffbe6; }
.log-info { color: #1890ff; background: #e6f7ff; }
.log-debug { color: #52c41a; background: #f6ffed; }
```

### 取消订阅

```javascript
// 示例：组件卸载时取消订阅
function unsubscribeLogs() {
  socket.send(JSON.stringify({
    type: 'UnsubscribeLogs'
  }));
}

// Vue/React 组件卸载时调用
onUnmounted(() => {
  unsubscribeLogs();
});
```

---

## 使用场景

1. **实时监控**
   - 监控 WebSocket 连接状态
   - 监控房间加入/离开事件
   - 监控消息收发情况

2. **问题排查**
   - 查看错误日志定位问题
   - 追踪特定用户的操作
   - 分析性能瓶颈

3. **运维管理**
   - 监控系统运行状态
   - 查看审计日志
   - 实时告警通知

---

## 注意事项

1. **权限控制**：仅 Admin 和 SuperAdmin 角色可订阅系统日志
2. **性能影响**：订阅大量日志可能影响客户端性能，建议按需过滤
3. **连接断开**：WebSocket 断开后订阅自动取消，重连后需重新订阅
4. **日志级别**：生产环境建议仅订阅 warn 和 error 级别日志

---

## 阶段归属

- **阶段 8 (配置化与运维管理)**: 系统日志流功能
- **阶段 8.4 (安全审计系统)**: 日志记录和监控
