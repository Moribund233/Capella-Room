import type { RouteRecordRaw } from 'vue-router'

/**
 * 路由配置
 * 用户客户端路由
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
      // 房间模块 - 主页面+子页面模式
      {
        path: 'room',
        name: 'Room',
        component: () => import('@/views/RoomView.vue'),
        redirect: '/room/list',
        meta: {
          title: '房间',
          requiresAuth: true,
          hasDock: true,
        },
        children: [
          {
            path: 'list',
            name: 'RoomList',
            component: () => import('@/pages/room/RoomListPage.vue'),
            meta: {
              title: '房间列表',
              requiresAuth: true,
            },
          },
          {
            path: 'chat/:id',
            name: 'RoomChat',
            component: () => import('@/pages/room/ChatPage.vue'),
            meta: {
              title: '聊天室',
              requiresAuth: true,
            },
          },
          {
            path: 'users/:id',
            name: 'RoomUsers',
            component: () => import('@/pages/room/UserListPage.vue'),
            meta: {
              title: '在线用户',
              requiresAuth: true,
            },
          },
        ],
      },
      {
        path: 'profile',
        name: 'Profile',
        component: () => import('@/views/ProfileView.vue'),
        meta: {
          title: '个人中心',
          requiresAuth: true,
        },
      },
      // 设置页面 - 主页面+子页面模式
      {
        path: 'setting',
        name: 'Setting',
        component: () => import('@/views/SettingView.vue'),
        redirect: '/setting/ui',
        meta: {
          title: '设置',
          requiresAuth: true,
          hasDock: true,
        },
        children: [
          {
            path: 'ui',
            name: 'SettingUI',
            component: () => import('@/pages/setting/UISettingsPanel.vue'),
            meta: {
              title: '界面设置',
              requiresAuth: true,
            },
          },
        ],
      },
      // Debug工具页面 - 主页面+子页面模式
      {
        path: 'debug',
        name: 'Debug',
        component: () => import('@/views/DebugView.vue'),
        redirect: '/debug/multi-user',
        meta: {
          title: '调试工具',
          requiresAuth: true,
          hasDock: true,
        },
        children: [
          {
            path: 'multi-user',
            name: 'MultiUserTest',
            component: () => import('@/pages/debug/MultiUserTest.vue'),
            meta: {
              title: '多用户测试',
              requiresAuth: true,
            },
          },
          {
            path: 'websocket',
            name: 'WebSocketTest',
            component: () => import('@/pages/debug/WebSocketTest.vue'),
            meta: {
              title: 'WebSocket测试',
              requiresAuth: true,
            },
          },
          {
            path: 'api',
            name: 'ApiTest',
            component: () => import('@/pages/debug/ApiTest.vue'),
            meta: {
              title: 'API测试',
              requiresAuth: true,
            },
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
