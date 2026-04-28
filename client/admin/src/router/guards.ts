import type { Router, RouteLocationNormalized } from 'vue-router'
import { useAuthStore } from '@/store'

/**
 * 设置路由守卫
 * @param router 路由实例
 */
export function setupRouterGuards(router: Router): void {
  router.beforeEach(async (to: RouteLocationNormalized) => {
    const authStore = useAuthStore()
    const requiresAuth = to.meta.requiresAuth as boolean

    // 初始化认证状态（如果尚未初始化）
    if (!authStore.isLoggedIn && authStore.accessToken) {
      await authStore.initAuth()
    }

    // 需要认证但未登录
    if (requiresAuth && !authStore.isLoggedIn) {
      return { name: 'Login', query: { redirect: to.fullPath } }
    }

    // 需要管理员权限但当前用户不是管理员
    if (requiresAuth && authStore.isLoggedIn && !authStore.isAdminRole) {
      // 清除认证信息并跳转到登录页
      await authStore.logout()
      return { name: 'Login', query: { redirect: to.fullPath, error: 'insufficient_permissions' } }
    }

    // 已登录用户访问登录页，重定向到首页
    if (to.name === 'Login' && authStore.isLoggedIn) {
      return { name: 'Home' }
    }

    return true
  })
}
