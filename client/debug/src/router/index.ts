import { createRouter, createWebHistory } from 'vue-router'
import { isAuthenticated } from '@/api/auth'
import Dashboard from '../views/Dashboard.vue'
import WebSocketTest from '../views/WebSocketTest.vue'
import ApiTest from '../views/ApiTest.vue'
import RoomManager from '../views/RoomManager.vue'
import UserManager from '../views/UserManager.vue'
import RoomTest from '../views/RoomTest.vue'
import Login from '../views/Login.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: Login,
      meta: { title: '登录', public: true }
    },
    {
      path: '/',
      name: 'dashboard',
      component: Dashboard,
      meta: { title: '仪表盘' }
    },
    {
      path: '/websocket',
      name: 'websocket',
      component: WebSocketTest,
      meta: { title: 'WebSocket测试' }
    },
    {
      path: '/api',
      name: 'api',
      component: ApiTest,
      meta: { title: 'API测试' }
    },
    {
      path: '/rooms',
      name: 'rooms',
      component: RoomManager,
      meta: { title: '房间管理' }
    },
    {
      path: '/users',
      name: 'users',
      component: UserManager,
      meta: { title: '用户管理' }
    },
    {
      path: '/room-test',
      name: 'room-test',
      component: RoomTest,
      meta: { title: '房间测试' }
    }
  ]
})

// 路由守卫
router.beforeEach((to) => {
  // 设置页面标题
  document.title = to.meta.title ? `${to.meta.title} - Seredeli Debug` : 'Seredeli Debug'

  // 检查是否需要认证
  if (!to.meta.public && !isAuthenticated()) {
    return '/login'
  } else if (to.path === '/login' && isAuthenticated()) {
    // 已登录用户访问登录页，重定向到首页
    return '/'
  }
  // 返回 undefined 表示继续导航
})

export default router
