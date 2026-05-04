# 接入真实后端

## 概述

SeredeliUI 在开发阶段使用模拟数据，同时预留了真实 API 的接入能力。本文档介绍如何将框架从模拟数据切换到真实后端 API。

## 架构设计

### 数据流

```
组件 → Store → API层 → 后端服务
   ↓
本地存储（localStorage）
```

### API 层结构

```
src/api/
├── index.ts      # API 导出
├── api.ts        # 通用请求函数
├── auth.ts       # 认证相关 API
└── ui.ts         # UI 配置相关 API
```

## 切换步骤

### 步骤1：配置环境变量

创建 `.env.production`：

```bash
# 应用环境
VITE_APP_ENV=production

# API 基础 URL
VITE_API_BASE_URL=https://your-api-server.com/api
```

### 步骤2：实现真实 API

#### 通用请求函数

`src/api/api.ts` 已实现通用的 `request` 函数：

```typescript
const BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api'

export async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${BASE_URL}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  })

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`)
  }

  return response.json()
}
```

#### 实现 UI 配置 API

修改 `src/api/ui.ts`：

```typescript
import { request } from './api'
import type { ApiResponse } from '@/types'
import type { UIConfigResponse, SaveUIConfigParams } from './ui'

export const uiApi = {
  /**
   * 获取用户云端 UI 配置
   */
  getUserConfig(): Promise<ApiResponse<UIConfigResponse>> {
    return request<ApiResponse<UIConfigResponse>>('/ui/config', {
      method: 'GET',
    })
  },

  /**
   * 保存用户云端 UI 配置
   */
  saveUserConfig(params: SaveUIConfigParams): Promise<ApiResponse<void>> {
    return request<ApiResponse<void>>('/ui/config', {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },

  /**
   * 重置用户云端 UI 配置
   */
  resetUserConfig(): Promise<ApiResponse<void>> {
    return request<ApiResponse<void>>('/ui/config', {
      method: 'DELETE',
    })
  },
}
```

#### 实现认证 API

修改 `src/api/auth.ts`：

```typescript
import { request } from './api'
import type { ApiResponse } from '@/types'
import type { LoginParams, LoginResult } from './auth'

export const authApi = {
  /**
   * 用户登录
   */
  login(params: LoginParams): Promise<ApiResponse<LoginResult>> {
    return request<ApiResponse<LoginResult>>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },

  /**
   * 用户注册
   */
  register(params: RegisterParams): Promise<ApiResponse<void>> {
    return request<ApiResponse<void>>('/auth/register', {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },
}
```

### 步骤3：增强请求函数（可选）

添加请求拦截器、Token 自动附加等功能：

```typescript
// src/api/api.ts
import { useAuthStore } from '@/store'

export async function request<T>(
  url: string,
  options: RequestInit = {}
): Promise<T> {
  const authStore = useAuthStore()
  
  // 自动附加 Token
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...options.headers as Record<string, string>,
  }
  
  if (authStore.token) {
    headers['Authorization'] = `Bearer ${authStore.token}`
  }

  try {
    const response = await fetch(`${BASE_URL}${url}`, {
      ...options,
      headers,
    })

    // 统一错误处理
    if (!response.ok) {
      if (response.status === 401) {
        // Token 过期，登出
        authStore.logout()
        window.location.href = '/login'
      }
      throw new Error(`HTTP error! status: ${response.status}`)
    }

    return response.json()
  } catch (error) {
    console.error('API request failed:', error)
    throw error
  }
}
```

### 步骤4：更新 Store

Store 层已经预留了云端配置的加载逻辑，只需确保环境变量设置正确：

```typescript
// src/store/ui.ts
async function loadCloudConfig(): Promise<void> {
  // 开发环境不使用云端配置
  if (isDev) {
    console.log('Dev mode: skip cloud config loading')
    return
  }

  isLoadingCloud.value = true
  try {
    const response = await uiApi.getUserConfig()
    if (response.success && response.data) {
      cloudConfig.value = response.data as Partial<UIConfig>
      lastSyncTime.value = new Date()
    }
  } catch (error) {
    console.error('Failed to load cloud config:', error)
  } finally {
    isLoadingCloud.value = false
  }
}
```

## 后端 API 规范

### 响应格式

所有 API 响应应遵循统一的格式：

```typescript
interface ApiResponse<T> {
  code: number      // 状态码，200 表示成功
  data: T          // 响应数据
  message: string  // 提示信息
  success: boolean // 是否成功
}
```

### 示例响应

成功响应：

```json
{
  "code": 200,
  "data": {
    "app": {
      "name": "MyApp"
    },
    "theme": {
      "name": "dark"
    }
  },
  "message": "获取成功",
  "success": true
}
```

错误响应：

```json
{
  "code": 401,
  "data": null,
  "message": "未授权",
  "success": false
}
```

### 需要实现的接口

#### 认证接口

| 接口 | 方法 | 说明 |
|------|------|------|
| /auth/login | POST | 用户登录 |
| /auth/register | POST | 用户注册 |

#### UI 配置接口

| 接口 | 方法 | 说明 |
|------|------|------|
| /ui/config | GET | 获取用户 UI 配置 |
| /ui/config | POST | 保存用户 UI 配置 |
| /ui/config | DELETE | 重置用户 UI 配置 |

## 开发环境 vs 生产环境

### 开发环境

```bash
# .env.development
VITE_APP_ENV=development
VITE_API_BASE_URL=http://localhost:8080/api
```

- 使用模拟数据
- 不加载云端配置
- 便于前端独立开发

### 生产环境

```bash
# .env.production
VITE_APP_ENV=production
VITE_API_BASE_URL=https://api.example.com/api
```

- 调用真实 API
- 加载云端配置
- 启用完整功能

## 常见问题

### Q: 如何同时支持开发和生产环境？

A: 使用环境变量控制：

```typescript
const isDev = import.meta.env.VITE_APP_ENV === 'development'

if (isDev) {
  // 开发环境逻辑
} else {
  // 生产环境逻辑
}
```

### Q: 如何处理跨域问题？

A: 配置 Vite 代理（开发环境）：

```typescript
// vite.config.ts
export default {
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
}
```

### Q: 如何调试 API 请求？

A: 在浏览器开发者工具中查看 Network 面板，或添加日志：

```typescript
request<T>(url: string, options: RequestInit = {}): Promise<T> {
  console.log('API Request:', url, options)
  // ...
}
```

## 示例后端实现

### FastAPI 示例

```python
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel

app = FastAPI()

class UIConfig(BaseModel):
    app: dict
    theme: dict

# 模拟数据库
user_configs = {}

@app.get("/api/ui/config")
async def get_config(user_id: str = "default"):
    return {
        "code": 200,
        "data": user_configs.get(user_id, {}),
        "message": "获取成功",
        "success": True
    }

@app.post("/api/ui/config")
async def save_config(config: UIConfig, user_id: str = "default"):
    user_configs[user_id] = config.dict()
    return {
        "code": 200,
        "data": None,
        "message": "保存成功",
        "success": True
    }
```

### Spring Boot 示例

```java
@RestController
@RequestMapping("/api/ui")
public class UIConfigController {
    
    @GetMapping("/config")
    public ApiResponse<UIConfig> getConfig(@RequestHeader("Authorization") String token) {
        // 获取用户配置
        UIConfig config = uiConfigService.getConfig(token);
        return ApiResponse.success(config);
    }
    
    @PostMapping("/config")
    public ApiResponse<Void> saveConfig(
        @RequestBody UIConfig config,
        @RequestHeader("Authorization") String token
    ) {
        uiConfigService.saveConfig(token, config);
        return ApiResponse.success(null, "保存成功");
    }
}
```

## 总结

切换到真实 API 的工作量很小，主要步骤：

1. 配置环境变量
2. 实现 API 函数（替换模拟数据）
3. 可选：增强请求函数

框架已经做好了充分的预留，切换过程平滑无痛。
