# 系统接口文档

> **认证要求**: 所有接口均无需认证

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 基础健康检查 |
| GET | `/health/detail` | 详细健康状态 |
| GET | `/health/ready` | 就绪检查 (Kubernetes) |
| GET | `/health/live` | 存活检查 (Kubernetes) |
| GET | `/api/version` | API 版本信息 |
| GET | `/api/config/client` | 客户端配置 |

---

## 基础健康检查

### 请求

```http
GET /health
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-15T08:30:00+00:00"
  }
}
```

### 说明

- 最简单的健康检查端点，仅检查服务是否存活
- 适用于负载均衡器的基础健康检查
- 响应迅速，不依赖外部服务

---

## 详细健康状态

### 请求

```http
GET /health/detail
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2024-01-15T08:30:00+00:00",
    "components": {
      "database": {
        "status": "healthy"
      },
      "websocket": {
        "status": "healthy",
        "connections": 42,
        "online_users": 15
      }
    }
  }
}
```

**降级状态 (200 OK)**

```json
{
  "success": true,
  "data": {
    "status": "degraded",
    "timestamp": "2024-01-15T08:30:00+00:00",
    "components": {
      "database": {
        "status": "unhealthy"
      },
      "websocket": {
        "status": "healthy",
        "connections": 42,
        "online_users": 15
      }
    }
  }
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `status` | string | 整体状态：`healthy` 或 `degraded` |
| `timestamp` | string | ISO 8601 格式时间戳 |
| `components.database.status` | string | 数据库状态：`healthy` / `unhealthy` |
| `components.websocket.status` | string | WebSocket 服务状态 |
| `components.websocket.connections` | number | 当前 WebSocket 连接数 |
| `components.websocket.online_users` | number | 当前在线用户数 |

### 说明

- 检查数据库连接状态
- 返回 WebSocket 连接统计信息
- 当数据库不可用时返回 `degraded` 状态

---

## 就绪检查 (Readiness Probe)

### 请求

```http
GET /health/ready
```

### 响应

**就绪 (200 OK)**

```json
{
  "success": true,
  "data": {
    "status": "ready",
    "timestamp": "2024-01-15T08:30:00+00:00"
  }
}
```

**未就绪 (503 Service Unavailable)**

```json
{
  "success": false,
  "data": {
    "status": "not_ready",
    "timestamp": "2024-01-15T08:30:00+00:00",
    "reason": "database unavailable"
  }
}
```

### 说明

- 用于 Kubernetes Readiness Probe
- 检查服务是否准备好接收流量
- 主要检查数据库连接状态
- 返回 503 时，Kubernetes 会将 Pod 从 Service 端点列表中移除

---

## 存活检查 (Liveness Probe)

### 请求

```http
GET /health/live
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "status": "alive",
    "timestamp": "2024-01-15T08:30:00+00:00"
  }
}
```

### 说明

- 用于 Kubernetes Liveness Probe
- 检查服务是否存活（未死锁、未卡死）
- 响应迅速，不依赖外部服务
- 返回失败时，Kubernetes 会重启 Pod

---

## API 版本信息

### 请求

```http
GET /api/version
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "version": "v1",
    "name": "Capella Room API",
    "description": "Real-time chat room API",
    "deprecated_routes": ["/api/*"],
    "recommended_routes": ["/api/v1/*"]
  }
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | string | 当前 API 版本 |
| `name` | string | API 名称 |
| `description` | string | API 描述 |
| `deprecated_routes` | array | 已弃用的路由前缀 |
| `recommended_routes` | array | 推荐使用的路由前缀 |

### 说明

- 用于客户端检测 API 版本兼容性
- 建议客户端在启动时调用此接口
- 当 API 升级时，可通过此接口通知客户端

---

## 客户端配置

### 请求

```http
GET /api/config/client
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "data": {
    "websocket": {
      "heartbeat_interval_secs": 30,
      "heartbeat_timeout_secs": 60,
      "auth_timeout_secs": 10
    },
    "reconnect": {
      "base_delay_ms": 1000,
      "max_delay_ms": 30000,
      "max_attempts": 10,
      "multiplier": 2
    },
    "upload": {
      "max_file_size": 10485760,
      "max_file_size_human": "10 MB"
    },
    "system": {
      "name": "Capella Room",
      "version": "0.1.0",
      "maintenance_mode": false,
      "maintenance_message": ""
    }
  }
}
```

### 响应字段说明

#### WebSocket 配置

| 字段 | 类型 | 说明 |
|------|------|------|
| `websocket.heartbeat_interval_secs` | number | 心跳发送间隔（秒） |
| `websocket.heartbeat_timeout_secs` | number | 心跳超时时间（秒） |
| `websocket.auth_timeout_secs` | number | WebSocket 认证超时（秒） |

#### 重连配置

| 字段 | 类型 | 说明 |
|------|------|------|
| `reconnect.base_delay_ms` | number | 基础重连延迟（毫秒） |
| `reconnect.max_delay_ms` | number | 最大重连延迟（毫秒） |
| `reconnect.max_attempts` | number | 最大重连次数 |
| `reconnect.multiplier` | number | 指数退避倍数 |

#### 上传配置

| 字段 | 类型 | 说明 |
|------|------|------|
| `upload.max_file_size` | number | 最大文件大小（字节） |
| `upload.max_file_size_human` | string | 人类可读的文件大小 |

#### 系统配置

| 字段 | 类型 | 说明 |
|------|------|------|
| `system.name` | string | 系统名称 |
| `system.version` | string | 系统版本 |
| `system.maintenance_mode` | boolean | 是否处于维护模式 |
| `system.maintenance_message` | string | 维护模式提示消息 |

### 说明

- 供前端应用获取服务端配置
- 配置值由服务端统一管理，支持动态更新
- 建议客户端在启动时获取此配置
- 维护模式可用于系统升级时的友好提示

---

## 使用示例

### cURL 示例

```bash
# 基础健康检查
curl http://localhost:8080/health

# 详细健康状态
curl http://localhost:8080/health/detail

# 就绪检查
curl http://localhost:8080/health/ready

# 存活检查
curl http://localhost:8080/health/live

# API 版本信息
curl http://localhost:8080/api/version

# 客户端配置
curl http://localhost:8080/api/config/client
```

### JavaScript 示例

```javascript
// 检查服务健康状态
async function checkHealth() {
  try {
    const response = await fetch('http://localhost:8080/health');
    const data = await response.json();
    return data.success && data.data.status === 'healthy';
  } catch (error) {
    return false;
  }
}

// 获取客户端配置
async function getClientConfig() {
  const response = await fetch('http://localhost:8080/api/config/client');
  const data = await response.json();
  
  if (data.success) {
    const config = data.data;
    
    // 检查维护模式
    if (config.system.maintenance_mode) {
      console.warn('系统维护中:', config.system.maintenance_message);
    }
    
    // 配置 WebSocket 心跳
    const heartbeatInterval = config.websocket.heartbeat_interval_secs * 1000;
    
    // 配置重连策略
    const reconnectConfig = {
      baseDelay: config.reconnect.base_delay_ms,
      maxDelay: config.reconnect.max_delay_ms,
      maxAttempts: config.reconnect.max_attempts,
      multiplier: config.reconnect.multiplier
    };
    
    return { heartbeatInterval, reconnectConfig };
  }
}
```

### Kubernetes 配置示例

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
    - name: capella-room
      image: capella-room:latest
      livenessProbe:
        httpGet:
          path: /health/live
          port: 8080
        initialDelaySeconds: 10
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /health/ready
          port: 8080
        initialDelaySeconds: 5
        periodSeconds: 5
```

---

## 错误响应

所有系统接口在发生错误时返回统一的错误格式：

```json
{
  "success": false,
  "data": null,
  "message": "错误描述信息"
}
```

### 可能的错误状态码

| 状态码 | 说明 |
|--------|------|
| 500 | 服务器内部错误 |
| 503 | 服务不可用（就绪检查失败） |

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*
