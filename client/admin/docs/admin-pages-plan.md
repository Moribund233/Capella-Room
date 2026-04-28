# 管理端页面开发规划

> 本文档描述管理后台需要完善的页面及开发规划

## 概述

管理后台采用**配置驱动**的设计理念，页面开发遵循以下规范：

- **简单单页**：直接编辑主视图
- **复杂单页**：主视图 + 组件模式
- **带子路由**：主视图 + 子页面（复杂子页面再使用组件）

## 已完成功能

| 模块 | 页面 | 路由 | 状态 |
|------|------|------|------|
| 用户管理 | 用户列表 | `/users` | ✅ 已完成 |
| 房间管理 | 房间列表 | `/rooms/list` | ✅ 已完成 |
| | 消息管理 | `/rooms/:id/messages` | ✅ 已完成 |
| | 数据分析 | `/rooms/:id/analytics` | ✅ 已完成 |
| 系统设置 | 界面设置 | `/setting/ui` | ✅ 已完成 |

## 待开发页面

### 1. 消息审核

**设计模式**：主视图 + 组件

**页面结构**：
```
src/
├── pages/
│   └── messages/
│       └── MessageListPage.vue          # 消息审核主页面
├── components/
│   └── messages/
│       ├── MessageTable.vue             # 消息表格
│       ├── MessageSearchForm.vue        # 搜索表单
│       └── MessageDetailModal.vue       # 消息详情弹窗
└── api/
    └── messages.ts                      # 消息审核API
```

**功能需求**：
- 展示所有房间的消息列表
- 支持按关键词、房间、用户、时间范围搜索
- 支持查看消息详情
- 支持删除违规消息
- 支持批量操作

**API接口**：
- `GET /api/v1/admin/messages` - 获取所有消息
- `DELETE /api/v1/admin/messages/:message_id` - 删除违规消息

**侧边栏配置**：
```typescript
{
  name: '消息审核',
  icon: 'Shield',
  path: '/messages',
}
```

---

### 2. 系统统计

**设计模式**：简单单页

**页面结构**：
```
src/
├── pages/
│   └── statistics/
│       └── StatisticsDashboard.vue      # 统计仪表盘
└── api/
    └── statistics.ts                    # 统计API
```

**功能需求**：
- 展示系统概览统计卡片
- 用户增长趋势图（折线图）
- 房间活跃度统计（柱状图）
- 消息量分布（饼图）
- 在线人数实时监控

**API接口**：
- `GET /api/v1/admin/stats` - 系统统计概览
- `GET /api/v1/admin/stats/activity` - 活跃度统计
- `GET /api/v1/admin/stats/performance` - 性能指标

**侧边栏配置**：
```typescript
{
  name: '系统统计',
  icon: 'BarChart3',
  path: '/statistics',
}
```

---

### 3. 审计系统

**设计模式**：主视图 + 子页面

**页面结构**：
```
src/
├── pages/
│   └── audit/
│       ├── AuditLogPage.vue             # 审计日志（默认）
│       ├── SecurityAlertPage.vue        # 安全告警
│       └── AlertRulePage.vue            # 告警规则
├── components/
│   └── audit/
│       ├── AuditLogTable.vue            # 审计日志表格
│       ├── AuditLogFilter.vue           # 日志筛选器
│       ├── AlertCard.vue                # 告警卡片
│       ├── AlertDetailModal.vue         # 告警详情
│       └── RuleEditor.vue               # 规则编辑器
└── api/
    └── audit.ts                         # 审计系统API
```

#### 3.1 审计日志页面

**功能需求**：
- 审计日志列表展示
- 支持按事件类型、严重级别、操作者、时间范围筛选
- 支持查看日志详情
- 支持导出审计日志

**API接口**：
- `GET /api/v1/admin/audit/logs` - 查询审计日志
- `GET /api/v1/admin/audit/logs/:id` - 获取日志详情
- `GET /api/v1/admin/audit/export` - 导出审计日志

#### 3.2 安全告警页面

**功能需求**：
- 告警列表展示（卡片或表格）
- 支持按状态、严重级别筛选
- 支持确认/解决/忽略告警
- 支持查看告警详情和相关日志

**API接口**：
- `GET /api/v1/admin/audit/alerts` - 获取安全告警列表
- `PUT /api/v1/admin/audit/alerts/:id/status` - 更新告警状态

#### 3.3 告警规则页面

**功能需求**：
- 规则列表展示
- 支持启用/禁用规则
- 支持修改规则配置（仅SuperAdmin）
- 支持查看规则详情

**API接口**：
- `GET /api/v1/admin/audit/rules` - 获取告警规则
- `PUT /api/v1/admin/audit/rules/:id` - 修改告警规则

**侧边栏配置**：
```typescript
{
  name: '审计系统',
  icon: 'ClipboardList',
  path: '/audit',
}
```

**DockBar配置**：
```typescript
audit: {
  enabled: true,
  position: 'bottom',
  offset: 24,
  items: [
    { key: 'logs', label: '审计日志', icon: 'ClipboardList', path: '/audit' },
    { key: 'alerts', label: '安全告警', icon: 'AlertTriangle', path: '/audit/alerts' },
    { key: 'rules', label: '告警规则', icon: 'Rule', path: '/audit/rules' },
  ],
}
```

---

### 4. IP安全管理

**设计模式**：主视图 + 组件

**页面结构**：
```
src/
├── pages/
│   └── security/
│       └── IPSecurityPage.vue           # IP安全管理主页面
├── components/
│   └── security/
│       ├── IPTable.vue                  # IP列表表格
│       ├── IPSearchForm.vue             # 搜索表单
│       ├── IPDetailModal.vue            # IP详情弹窗
│       └── IPAddModal.vue               # 添加IP弹窗
└── api/
    └── security.ts                      # IP安全API
```

**功能需求**：
- IP黑白名单列表展示
- 支持按IP地址、列表类型搜索
- 支持添加/编辑/删除IP条目
- 支持批量添加IP
- 支持设置过期时间
- 白名单模式开关（仅SuperAdmin）

**API接口**：
- `GET /api/v1/admin/security/ip-list` - 查询IP列表
- `POST /api/v1/admin/security/ip-list` - 添加IP到列表
- `POST /api/v1/admin/security/ip-list/batch` - 批量添加IP
- `PUT /api/v1/admin/security/ip-list/:id` - 更新IP条目
- `DELETE /api/v1/admin/security/ip-list/:id` - 移除IP
- `POST /api/v1/admin/security/whitelist-mode` - 设置白名单模式

**侧边栏配置**：
```typescript
{
  name: 'IP安全',
  icon: 'ShieldCheck',
  path: '/security',
}
```

---

### 5. 系统配置

**设计模式**：简单单页

**页面结构**：
```
src/
├── pages/
│   └── setting/
│       └── ConfigSettingsPage.vue       # 系统配置页面
└── api/
    └── config.ts                        # 配置API
```

**功能需求**：
- 系统配置项列表
- 支持按分类筛选
- 支持编辑配置值（仅SuperAdmin）
- 支持重置配置到默认值
- 显示配置项说明和类型

**API接口**：
- `GET /api/v1/admin/configs` - 获取所有配置项
- `PUT /api/v1/admin/configs/:key` - 修改配置项
- `POST /api/v1/admin/configs/reset` - 重置配置到默认值

**DockBar配置**：
```typescript
setting: {
  enabled: true,
  position: 'bottom',
  offset: 24,
  items: [
    { key: 'ui', label: '界面设置', icon: 'Palette', path: '/setting/ui' },
    { key: 'config', label: '系统配置', icon: 'Settings', path: '/setting/config' },
    { key: 'redis', label: 'Redis状态', icon: 'Database', path: '/setting/redis' },
  ],
}
```

---

### 6. Redis状态

**设计模式**：简单单页

**页面结构**：
```
src/
├── pages/
│   └── setting/
│       └── RedisStatusPage.vue          # Redis状态页面
└── api/
    └── redis.ts                         # Redis API
```

**功能需求**：
- Redis连接状态展示
- Redis统计信息展示
- 支持刷新Redis连接（仅SuperAdmin）
- 配置同步状态展示
- 支持触发配置同步（仅SuperAdmin）

**API接口**：
- `GET /api/v1/admin/redis/status` - 获取Redis连接状态
- `GET /api/v1/admin/redis/stats` - 获取Redis统计信息
- `POST /api/v1/admin/redis/refresh` - 刷新Redis连接
- `GET /api/v1/admin/config/sync/status` - 获取配置同步状态
- `POST /api/v1/admin/config/sync` - 触发配置同步

---

## 路由配置

```typescript
// src/router/routes.ts
export const routes: RouteRecordRaw[] = [
  // ... 其他路由
  {
    path: 'messages',
    name: 'MessageManagement',
    component: () => import('@/views/MessageManagementView.vue'),
    meta: { title: '消息审核', requiresAuth: true },
  },
  {
    path: 'statistics',
    name: 'Statistics',
    component: () => import('@/views/StatisticsView.vue'),
    meta: { title: '系统统计', requiresAuth: true },
  },
  {
    path: 'audit',
    name: 'Audit',
    component: () => import('@/views/AuditView.vue'),
    redirect: '/audit/logs',
    meta: { title: '审计系统', requiresAuth: true },
    children: [
      {
        path: 'logs',
        name: 'AuditLogs',
        component: () => import('@/pages/audit/AuditLogPage.vue'),
        meta: { title: '审计日志', requiresAuth: true },
      },
      {
        path: 'alerts',
        name: 'SecurityAlerts',
        component: () => import('@/pages/audit/SecurityAlertPage.vue'),
        meta: { title: '安全告警', requiresAuth: true },
      },
      {
        path: 'rules',
        name: 'AlertRules',
        component: () => import('@/pages/audit/AlertRulePage.vue'),
        meta: { title: '告警规则', requiresAuth: true },
      },
    ],
  },
  {
    path: 'security',
    name: 'Security',
    component: () => import('@/views/SecurityView.vue'),
    meta: { title: 'IP安全', requiresAuth: true },
  },
  {
    path: 'setting',
    name: 'Setting',
    component: () => import('@/views/SettingView.vue'),
    redirect: '/setting/ui',
    meta: { title: '设置', requiresAuth: true },
    children: [
      {
        path: 'ui',
        name: 'SettingUI',
        component: () => import('@/pages/setting/UISettingsPanel.vue'),
        meta: { title: '界面设置', requiresAuth: true },
      },
      {
        path: 'config',
        name: 'SettingConfig',
        component: () => import('@/pages/setting/ConfigSettingsPage.vue'),
        meta: { title: '系统配置', requiresAuth: true, requiresSuperAdmin: true },
      },
      {
        path: 'redis',
        name: 'SettingRedis',
        component: () => import('@/pages/setting/RedisStatusPage.vue'),
        meta: { title: 'Redis状态', requiresAuth: true },
      },
    ],
  },
]
```

---

## 开发优先级建议

1. **高优先级**
   - 系统统计（展示系统运行状态，基础功能）
   - 消息审核（核心管理功能）

2. **中优先级**
   - 审计系统（安全相关，包含日志、告警、规则）
   - IP安全管理（安全防护功能）

3. **低优先级**
   - 系统配置（SuperAdmin功能，使用频率较低）
   - Redis状态（运维功能，使用频率较低）

---

## 权限控制

部分页面需要SuperAdmin权限：

| 页面 | 权限要求 |
|------|----------|
| 系统配置 | SuperAdmin |
| 告警规则修改 | SuperAdmin |
| Redis刷新连接 | SuperAdmin |
| 配置同步 | SuperAdmin |
| 白名单模式设置 | SuperAdmin |

在路由meta中添加权限标记：
```typescript
meta: {
  title: '系统配置',
  requiresAuth: true,
  requiresSuperAdmin: true,
}
```

---

## 参考文档

- [页面开发指南](./page-development.md) - 页面开发基础规范
- [后端集成指南](./backend-integration.md) - API集成说明
- [QuickBar开发指南](./quickbar-development.md) - Dock栏配置说明
