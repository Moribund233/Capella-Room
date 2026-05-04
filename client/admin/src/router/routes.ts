import type { RouteRecordRaw } from 'vue-router'

/**
 * 路由配置
 */
export const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/LoginView.vue'),
    meta: {
      title: '登录',
      requiresAuth: false,
    },
  },
  {
    path: '/',
    name: 'Layout',
    component: () => import('@/components/layout/MainLayout.vue'),
    redirect: '/home',
    meta: {
      requiresAuth: true,
    },
    children: [
      {
        path: 'home',
        name: 'Home',
        component: () => import('@/views/HomeView.vue'),
        meta: {
          title: '首页',
          requiresAuth: true,
        },
      },
      {
        path: 'users',
        name: 'UserManagement',
        component: () => import('@/views/UserManagementView.vue'),
        meta: {
          title: '用户管理',
          requiresAuth: true,
        },
      },
      {
        path: 'rooms',
        name: 'RoomManagement',
        component: () => import('@/views/RoomManagementView.vue'),
        redirect: '/rooms/list',
        meta: {
          title: '房间管理',
          requiresAuth: true,
        },
        children: [
          {
            path: 'list',
            name: 'RoomList',
            component: () => import('@/pages/rooms/RoomListPage.vue'),
            meta: { title: '房间列表', requiresAuth: true },
          },
          {
            path: ':id/messages',
            name: 'RoomMessages',
            component: () => import('@/pages/rooms/RoomMessagesPage.vue'),
            meta: { title: '消息管理', requiresAuth: true },
          },
          {
            path: ':id/analytics',
            name: 'RoomAnalytics',
            component: () => import('@/pages/rooms/RoomAnalyticsPage.vue'),
            meta: { title: '数据分析', requiresAuth: true },
          },
        ],
      },
      {
        path: 'messages',
        name: 'MessageManagement',
        component: () => import('@/views/MessageManagementView.vue'),
        meta: {
          title: '消息审核',
          requiresAuth: true,
        },
      },
      {
        path: 'statistics',
        name: 'Statistics',
        component: () => import('@/views/StatisticsView.vue'),
        meta: {
          title: '系统统计',
          requiresAuth: true,
        },
      },
      {
        path: 'audit',
        name: 'Audit',
        component: () => import('@/views/AuditView.vue'),
        redirect: '/audit/logs',
        meta: {
          title: '审计系统',
          requiresAuth: true,
        },
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
            meta: { title: '告警规则', requiresAuth: true, requiresSuperAdmin: true },
          },
        ],
      },
      {
        path: 'security',
        name: 'Security',
        component: () => import('@/views/IPSecurityView.vue'),
        meta: {
          title: 'IP安全',
          requiresAuth: true,
        },
      },
      {
        path: 'setting',
        name: 'Setting',
        component: () => import('@/views/SettingView.vue'),
        redirect: '/setting/ui',
        meta: {
          title: '设置',
          requiresAuth: true,
        },
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
            meta: { title: '系统配置', requiresAuth: true },
          },
          {
            path: 'redis',
            name: 'SettingRedis',
            component: () => import('@/pages/setting/RedisStatusPage.vue'),
            meta: { title: 'Redis状态', requiresAuth: true },
          },
        ],
      },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/views/NotFoundView.vue'),
    meta: {
      title: '页面未找到',
      requiresAuth: false,
    },
  },
]
