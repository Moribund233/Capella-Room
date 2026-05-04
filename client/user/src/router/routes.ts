import type { RouteRecordRaw } from 'vue-router'
import { ROUTE_PATHS, ROUTE_NAMES } from '@/constants'

export const routes: RouteRecordRaw[] = [
  {
    path: ROUTE_PATHS.LOGIN,
    name: ROUTE_NAMES.LOGIN,
    component: () => import('@/views/LoginView.vue'),
    meta: { public: true },
  },
  {
    path: ROUTE_PATHS.REGISTER,
    name: ROUTE_NAMES.REGISTER,
    component: () => import('@/views/RegisterView.vue'),
    meta: { public: true },
  },
  {
    path: ROUTE_PATHS.CHAT,
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        name: ROUTE_NAMES.CHAT,
        component: () => import('@/views/ChatView.vue'),
      },
      {
        path: 'room/:roomId',
        name: ROUTE_NAMES.CHAT_ROOM,
        component: () => import('@/views/ChatRoomView.vue'),
      },
    ],
  },
  {
    path: ROUTE_PATHS.PROFILE,
    name: ROUTE_NAMES.PROFILE,
    component: () => import('@/views/ProfileView.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: ROUTE_PATHS.SETTINGS,
    name: ROUTE_NAMES.SETTINGS,
    component: () => import('@/views/SettingsView.vue'),
    meta: { requiresAuth: true },
  },
  {
    path: '/:pathMatch(.*)*',
    name: ROUTE_NAMES.NOT_FOUND,
    component: () => import('@/views/NotFoundView.vue'),
    meta: { public: true },
  },
]
