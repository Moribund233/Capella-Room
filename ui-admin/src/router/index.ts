import { createRouter, createWebHistory } from 'vue-router'
import MainLayout from '@/components/layouts/MainLayout.vue'
import { useUserStore } from '@/stores/user'

/**
 * 路由配置
 * 定义应用的所有路由
 */

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: MainLayout,
      redirect: '/dashboard',
      meta: {
        requiresAuth: true,
      },
      children: [
        {
          path: 'dashboard',
          name: 'Dashboard',
          component: () => import('@/views/dashboard/index.vue'),
          meta: {
            title: '仪表盘',
            icon: 'HomeFilled',
            requiresAuth: true,
          },
        },
        {
          path: 'users',
          name: 'Users',
          component: () => import('@/views/users/index.vue'),
          meta: {
            title: '用户管理',
            icon: 'UserFilled',
            requiresAuth: true,
          },
        },
        {
          path: 'monitor',
          name: 'Monitor',
          component: () => import('@/views/monitor/index.vue'),
          meta: {
            title: '系统监控',
            icon: 'Monitor',
            requiresAuth: true,
          },
        },
        {
          path: 'documents',
          name: 'Documents',
          component: () => import('@/views/documents/index.vue'),
          meta: {
            title: '文档管理',
            icon: 'Document',
            requiresAuth: true,
          },
        },
        {
          path: 'settings',
          name: 'Settings',
          component: () => import('@/views/settings/index.vue'),
          meta: {
            title: '系统设置',
            icon: 'Setting',
            requiresAuth: true,
          },
        },
      ],
    },
    {
      path: '/login',
      name: 'Login',
      component: () => import('@/views/login/index.vue'),
      meta: {
        title: '登录',
        public: true,
      },
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'NotFound',
      component: () => import('@/views/error/404.vue'),
      meta: {
        title: '页面不存在',
      },
    },
  ],
})

/**
 * 路由守卫
 * 处理认证检查和页面标题设置
 */
router.beforeEach((to, from, next) => {
  // 设置页面标题
  const title = to.meta.title as string
  if (title) {
    document.title = `${title} - Seredeli Admin`
  } else {
    document.title = 'Seredeli Admin'
  }

  // 获取用户状态
  const userStore = useUserStore()
  const isLoggedIn = userStore.isLoggedIn

  // 检查是否需要认证
  const requiresAuth = to.matched.some((record) => record.meta.requiresAuth)
  const isPublicRoute = to.meta.public === true

  // 已登录用户访问登录页，重定向到仪表盘
  if (isLoggedIn && to.path === '/login') {
    next('/dashboard')
    return
  }

  // 需要认证但未登录，重定向到登录页
  if (requiresAuth && !isLoggedIn) {
    next({
      path: '/login',
      query: { redirect: to.fullPath },
    })
    return
  }

  // 其他情况正常导航
  next()
})

export default router
