import type { RouteRecordRaw } from 'vue-router'
import { MainLayout } from '@/layouts'

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'landing',
    component: () => import('@/views/LandingView.vue'),
    meta: { public: true },
  },
  {
    path: '/login',
    name: 'login',
    component: () => import('@/views/LoginView.vue'),
    meta: { public: true },
  },
  {
    path: '/register',
    name: 'register',
    component: () => import('@/views/RegisterView.vue'),
    meta: { public: true },
  },
  {
    path: '/invite/:code',
    name: 'invite',
    component: () => import('@/views/InviteValidationView.vue'),
    meta: { public: true },
  },
  // 需要认证的页面使用 MainLayout
  {
    path: '/',
    component: MainLayout,
    meta: { requiresAuth: true },
    children: [
      {
        path: 'app',
        name: 'app',
        component: () => import('@/views/AppView.vue'),
      },
      {
        path: 'profile',
        name: 'profile',
        component: () => import('@/views/ProfileView.vue'),
      },
      {
        path: 'discover',
        name: 'discover',
        component: () => import('@/views/DiscoverView.vue'),
      },
      {
        path: 'friends',
        name: 'friends',
        component: () => import('@/views/FriendsView.vue'),
      },

    ],
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@/views/NotFoundView.vue'),
    meta: { public: true },
  },
]
