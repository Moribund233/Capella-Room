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
        path: 'example',
        name: 'Example',
        component: () => import('@/views/ExampleView.vue'),
        redirect: '/example/overview',
        meta: {
          title: '示例',
          requiresAuth: true,
        },
        children: [
          {
            path: 'overview',
            name: 'ExampleOverview',
            component: () => import('@/pages/example/OverviewPanel.vue'),
            meta: { title: '概览', requiresAuth: true },
          },
          {
            path: 'icons',
            name: 'ExampleIcons',
            component: () => import('@/pages/example/IconPickerPanel.vue'),
            meta: { title: '图标', requiresAuth: true },
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
