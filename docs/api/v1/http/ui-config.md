# UI 配置接口文档

> **认证要求**: 所有接口均需要认证（需要携带 Access Token）
> 
> **适用说明**: 此接口专为 CapellaUI 前端框架设计，用于云端同步用户界面配置。其他客户端开发可选择性实现。

## 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/ui/config` | 获取用户 UI 配置 |
| POST | `/api/v1/ui/config` | 保存用户 UI 配置 |
| DELETE | `/api/v1/ui/config` | 重置用户 UI 配置 |

---

## 配置结构说明

UI 配置采用模块化设计，支持以下配置项：

### 配置模块

| 模块 | 说明 | 适用场景 |
|------|------|---------|
| `app` | 应用配置 | 应用名称、Logo、版本 |
| `theme` | 主题配置 | 主题名称、颜色方案 |
| `sidebar` | 侧边栏配置 | 导航菜单、路由 |
| `quickbar` | 快捷栏配置 | 快捷按钮、操作菜单 |
| `dock` | Dock 栏配置 | 页面 Dock、快捷入口 |

---

## 获取用户 UI 配置

获取当前用户的 UI 配置，如果用户没有保存过配置，则返回默认配置。

### 请求

```http
GET /api/v1/ui/config
Authorization: Bearer {access_token}
```

### 响应

**成功 - 已有配置 (200 OK)**

```json
{
  "success": true,
  "data": {
    "app": {
      "name": "Capella Room",
      "logo": "/logo.svg",
      "version": "1.0.0"
    },
    "theme": {
      "name": "light"
    },
    "sidebar": {
      "items": [
        {
          "name": "Dashboard",
          "icon": "LayoutDashboard",
          "path": "/dashboard"
        },
        {
          "name": "Chat Rooms",
          "icon": "MessageSquare",
          "path": "/rooms"
        },
        {
          "name": "Users",
          "icon": "Users",
          "path": "/users"
        },
        {
          "name": "Settings",
          "icon": "Settings",
          "path": "/settings"
        }
      ]
    },
    "quickbar": [
      {
        "key": "notifications",
        "display": "visible",
        "type": "action",
        "icon": "Bell",
        "icon_alt": "BellDot",
        "label": "通知",
        "badge": {
          "count": 5,
          "max": 99
        }
      },
      {
        "key": "user_menu",
        "display": "dropdown",
        "type": "menu",
        "icon": "User",
        "label": "用户菜单",
        "children": [
          {
            "key": "profile",
            "label": "个人资料",
            "icon": "UserCircle"
          },
          {
            "key": "settings",
            "label": "设置",
            "icon": "Settings"
          },
          {
            "key": "logout",
            "label": "退出登录",
            "icon": "LogOut",
            "disabled": false
          }
        ]
      }
    ],
    "dock": {
      "dashboard": {
        "enabled": true,
        "position": "bottom",
        "offset": 0,
        "items": [
          {
            "key": "home",
            "label": "首页",
            "icon": "Home",
            "path": "/dashboard",
            "disabled": false
          },
          {
            "key": "analytics",
            "label": "分析",
            "icon": "BarChart3",
            "path": "/dashboard/analytics"
          }
        ]
      },
      "rooms": {
        "enabled": true,
        "position": "left",
        "offset": 20,
        "items": [
          {
            "key": "all_rooms",
            "label": "所有房间",
            "icon": "List",
            "path": "/rooms"
          },
          {
            "key": "favorites",
            "label": "收藏",
            "icon": "Star",
            "path": "/rooms/favorites"
          }
        ]
      }
    }
  }
}
```

**成功 - 无配置返回默认 (200 OK)**

```json
{
  "success": true,
  "data": {}
}
```

### 响应字段说明

#### AppConfig (应用配置)

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 应用名称 |
| `logo` | string | Logo URL |
| `version` | string | 应用版本 |

#### ThemeConfig (主题配置)

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 主题名称：`light` / `dark` / `auto` |

#### SidebarConfig (侧边栏配置)

| 字段 | 类型 | 说明 |
|------|------|------|
| `items` | array | 侧边栏菜单项列表 |

##### SidebarItemConfig (侧边栏菜单项)

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 菜单名称 |
| `icon` | string | 图标名称（Lucide 图标） |
| `path` | string | 路由路径 |

#### QuickBarConfig (快捷栏配置)

`quickbar` 是数组，每个元素是一个快捷按钮配置：

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | string | 唯一标识 |
| `display` | string | 显示方式：`visible` (直接显示) / `dropdown` (下拉菜单) |
| `type` | string | 类型：`action` (点击动作) / `menu` (菜单) |
| `icon` | string | 图标名称 |
| `icon_alt` | string | 替代图标（用于状态切换，如 Bell/BellDot） |
| `label` | string | 按钮标签 |
| `badge` | object | 角标配置 |
| `children` | array | 子菜单项（type=menu 时有效） |

##### BadgeConfig (角标配置)

| 字段 | 类型 | 说明 |
|------|------|------|
| `count` | number | 角标数字 |
| `max` | number | 最大显示数字，超过显示 `{max}+` |

##### QuickChildItemConfig (子菜单项)

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | string | 唯一标识 |
| `label` | string | 菜单标签 |
| `icon` | string | 图标名称 |
| `disabled` | boolean | 是否禁用 |

#### DockConfig (Dock 栏配置)

`dock` 是对象，键为页面名称，值为该页面的 Dock 配置：

| 字段 | 类型 | 说明 |
|------|------|------|
| `enabled` | boolean | 是否启用 |
| `position` | string | 位置：`bottom` / `left` / `right` |
| `offset` | number | 偏移量（像素） |
| `items` | array | Dock 项目列表 |

##### DockItemConfig (Dock 项目)

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | string | 唯一标识 |
| `label` | string | 标签 |
| `icon` | string | 图标名称 |
| `path` | string | 路由路径 |
| `disabled` | boolean | 是否禁用 |

---

## 保存用户 UI 配置

保存用户的 UI 配置到云端，支持增量更新（只传需要更新的模块）。

### 请求

```http
POST /api/v1/ui/config
Authorization: Bearer {access_token}
Content-Type: application/json
```

### 请求体

```json
{
  "theme": {
    "name": "dark"
  },
  "sidebar": {
    "items": [
      {
        "name": "Dashboard",
        "icon": "LayoutDashboard",
        "path": "/dashboard"
      },
      {
        "name": "Chat Rooms",
        "icon": "MessageSquare",
        "path": "/rooms"
      }
    ]
  }
}
```

### 请求字段说明

- 支持部分更新，只传入需要保存的模块
- 未传入的模块保持原有配置不变
- 传入 `null` 会删除该模块的配置

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "message": "配置已保存"
}
```

**失败 - 配置格式错误 (400 Bad Request)**

```json
{
  "success": false,
  "code": "VALIDATION_ERROR",
  "error": "请求参数错误",
  "message": "配置格式无效"
}
```

---

## 重置用户 UI 配置

删除用户的云端 UI 配置，恢复默认设置。

### 请求

```http
DELETE /api/v1/ui/config
Authorization: Bearer {access_token}
```

### 响应

**成功 (200 OK)**

```json
{
  "success": true,
  "message": "配置已重置"
}
```

### 说明

- 删除云端配置后，下次获取配置将返回默认空配置
- 客户端应回退到本地默认配置或应用内置配置

---

## 使用示例

### cURL 示例

```bash
# 获取 UI 配置
curl -X GET "http://localhost:3000/api/v1/ui/config" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

# 保存主题配置
curl -X POST "http://localhost:3000/api/v1/ui/config" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "theme": {
      "name": "dark"
    }
  }'

# 保存完整侧边栏配置
curl -X POST "http://localhost:3000/api/v1/ui/config" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "sidebar": {
      "items": [
        {
          "name": "Dashboard",
          "icon": "LayoutDashboard",
          "path": "/dashboard"
        },
        {
          "name": "Chat Rooms",
          "icon": "MessageSquare",
          "path": "/rooms"
        }
      ]
    }
  }'

# 保存 QuickBar 配置
curl -X POST "http://localhost:3000/api/v1/ui/config" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "quickbar": [
      {
        "key": "notifications",
        "display": "visible",
        "type": "action",
        "icon": "Bell",
        "label": "通知",
        "badge": {
          "count": 0,
          "max": 99
        }
      }
    ]
  }'

# 重置 UI 配置
curl -X DELETE "http://localhost:3000/api/v1/ui/config" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### JavaScript 示例

```javascript
// 获取 UI 配置
async function getUIConfig() {
  const response = await fetch(
    'http://localhost:3000/api/v1/ui/config',
    {
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data.data;
  } else {
    throw new Error(data.message);
  }
}

// 保存 UI 配置（支持部分更新）
async function saveUIConfig(config) {
  const response = await fetch(
    'http://localhost:3000/api/v1/ui/config',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(config)
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data;
  } else {
    throw new Error(data.message);
  }
}

// 重置 UI 配置
async function resetUIConfig() {
  const response = await fetch(
    'http://localhost:3000/api/v1/ui/config',
    {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token')}`
      }
    }
  );
  
  const data = await response.json();
  
  if (data.success) {
    return data;
  } else {
    throw new Error(data.message);
  }
}

// 切换主题
async function switchTheme(themeName) {
  await saveUIConfig({
    theme: {
      name: themeName // 'light' | 'dark' | 'auto'
    }
  });
}

// 更新侧边栏
async function updateSidebar(items) {
  await saveUIConfig({
    sidebar: {
      items: items
    }
  });
}

// 更新 QuickBar 角标
async function updateQuickBarBadge(key, count) {
  // 先获取当前配置
  const config = await getUIConfig();
  
  // 更新指定项的角标
  const quickbar = config.quickbar || [];
  const item = quickbar.find(item => item.key === key);
  
  if (item) {
    item.badge = { count, max: 99 };
    await saveUIConfig({ quickbar });
  }
}

// Vue 组合式函数示例
import { ref, onMounted } from 'vue';

export function useUIConfig() {
  const config = ref({});
  const loading = ref(false);
  const error = ref(null);
  
  // 加载配置
  const loadConfig = async () => {
    loading.value = true;
    error.value = null;
    
    try {
      config.value = await getUIConfig();
    } catch (err) {
      error.value = err.message;
      // 加载失败时使用本地默认配置
      config.value = getDefaultConfig();
    } finally {
      loading.value = false;
    }
  };
  
  // 保存配置
  const saveConfig = async (newConfig) => {
    loading.value = true;
    
    try {
      await saveUIConfig(newConfig);
      // 合并到本地配置
      config.value = { ...config.value, ...newConfig };
    } catch (err) {
      error.value = err.message;
      throw err;
    } finally {
      loading.value = false;
    }
  };
  
  // 重置配置
  const resetConfig = async () => {
    loading.value = true;
    
    try {
      await resetUIConfig();
      config.value = getDefaultConfig();
    } catch (err) {
      error.value = err.message;
    } finally {
      loading.value = false;
    }
  };
  
  onMounted(loadConfig);
  
  return {
    config,
    loading,
    error,
    loadConfig,
    saveConfig,
    resetConfig
  };
}

// 默认配置
function getDefaultConfig() {
  return {
    app: {
      name: 'Capella Room',
      logo: '/logo.svg',
      version: '1.0.0'
    },
    theme: {
      name: 'light'
    },
    sidebar: {
      items: [
        { name: 'Dashboard', icon: 'LayoutDashboard', path: '/dashboard' },
        { name: 'Chat Rooms', icon: 'MessageSquare', path: '/rooms' },
        { name: 'Users', icon: 'Users', path: '/users' },
        { name: 'Settings', icon: 'Settings', path: '/settings' }
      ]
    },
    quickbar: [],
    dock: {}
  };
}
```

---

## 配置同步策略

### 推荐实现

1. **启动时加载**: 应用启动时从云端加载配置
2. **本地缓存**: 将配置缓存到 localStorage，下次快速加载
3. **增量更新**: 只保存变更的配置模块，减少数据传输
4. **冲突处理**: 云端配置优先，或提供合并策略
5. **离线支持**: 离线时使用本地缓存，联网后同步

### 同步流程

```
┌─────────────────┐
│   应用启动      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     存在     ┌─────────────────┐
│ 读取本地缓存    │─────────────▶│ 应用配置        │
└────────┬────────┘              └─────────────────┘
         │ 不存在
         ▼
┌─────────────────┐     成功     ┌─────────────────┐
│ 请求云端配置    │─────────────▶│ 保存到本地缓存  │
└────────┬────────┘              └────────┬────────┘
         │ 失败                           │
         ▼                                ▼
┌─────────────────┐              ┌─────────────────┐
│ 使用默认配置    │              │ 应用配置        │
└─────────────────┘              └─────────────────┘
```

---

## 错误码汇总

### HTTP 状态码

| HTTP 状态码 | 错误场景 | 说明 |
|------------|---------|------|
| 200 | 请求成功 | 操作成功 |
| 400 | 请求参数错误 | 配置格式无效 |
| 401 | 认证失败 | Token 无效或过期 |
| 500 | 服务器错误 | 内部服务器错误 |

### 业务错误码 (code)

| 错误码 | HTTP 状态码 | 说明 | 处理建议 |
|--------|------------|------|---------|
| `VALIDATION_ERROR` | 400 | 配置格式无效 | 检查配置结构是否符合要求 |
| `AUTH_ERROR` | 401 | 认证失败 | 检查 Token 是否过期 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 | 稍后重试或联系管理员 |

---

## 与 CapellaUI 的集成

此接口专为 [CapellaUI](../../../../../client/admin/README.md) 设计，支持：

- **配置驱动 UI**: 通过云端配置动态渲染界面
- **多端同步**: 同一账号在不同设备上配置同步
- **个性化定制**: 用户可自定义侧边栏、快捷栏、Dock 等
- **主题切换**: 支持浅色/深色主题云端保存

### 前端集成示例

```typescript
// CapellaUI 配置示例
export const useCapellaConfig = () => {
  const { config, saveConfig } = useUIConfig();
  
  // 将云端配置转换为 CapellaUI 格式
  const uiConfig = computed(() => ({
    app: config.value.app,
    theme: config.value.theme,
    sidebar: config.value.sidebar?.items || [],
    quickBar: config.value.quickbar || [],
    dock: config.value.dock || {}
  }));
  
  return {
    uiConfig,
    updateConfig: saveConfig
  };
};
```

---

*文档版本: 1.0.0*  
*最后更新: 2026-04-26*  
*适用客户端: CapellaUI 管理后台*
