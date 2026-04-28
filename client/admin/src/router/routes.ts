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
            component: () => import('@/pages/RoomListPage.vue'),
            meta: { title: '房间列表', requiresAuth: true },
          },
          {
            path: ':id/messages',
            name: 'RoomMessages',
            component: () => import('@/pages/RoomMessagesPage.vue'),
            meta: { title: '消息管理', requiresAuth: true },
          },
          {
            path: ':id/analytics',
            name: 'RoomAnalytics',
            component: () => import('@/pages/RoomAnalyticsPage.vue'),
            meta: { title: '数据分析', requiresAuth: true },
          },
        ],
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
