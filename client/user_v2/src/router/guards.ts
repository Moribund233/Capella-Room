import type { Router } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const publicRoutes = ['login', 'register', 'landing', 'not-found']

export function setupRouterGuards(router: Router) {
  router.beforeEach(async (to, from) => {
    const authStore = useAuthStore()
    const isPublic = publicRoutes.includes(to.name as string) || to.meta?.public

    // 使用 isAuthenticated 计算属性判断登录状态
    const isAuthenticated = authStore.isAuthenticated

    // 未登录且访问非公开页面，重定向到登录页
    if (!isPublic && !isAuthenticated) {
      return { name: 'login', query: { redirect: to.fullPath } }
    }

    // 已登录且访问登录页，重定向到首页
    if (isPublic && isAuthenticated && to.name === 'login' && from.name !== 'app') {
      return { name: 'app' }
    }
  })
}
