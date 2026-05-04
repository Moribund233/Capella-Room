/**
 * 路由守卫
 */

import type { Router, RouteLocationNormalized } from 'vue-router'
import { isAuthenticated } from '@/api/auth'
import { useAuthStore } from '@/store/auth'
import { TOKEN_EXPIRED_EVENT } from '@/api/client'

/**
 * 设置路由守卫
 */
export function setupRouterGuards(router: Router): void {
  // 全局前置守卫
  router.beforeEach(async (to: RouteLocationNormalized) => {
    // 设置页面标题
    const title = to.meta.title as string
    if (title) {
      document.title = `${title} - 聊天室`
    }

    // 检查是否需要认证
    const requiresAuth = to.meta.requiresAuth !== false

    if (requiresAuth) {
      // 检查是否已登录
      if (!isAuthenticated()) {
        // 未登录，重定向到登录页
        return {
          name: 'Login',
          query: { redirect: to.fullPath },
        }
      }

      // 已登录，获取用户信息
      const authStore = useAuthStore()
      if (!authStore.user) {
        const success = await authStore.fetchCurrentUser()
        if (!success) {
          return {
            name: 'Login',
            query: { redirect: to.fullPath },
          }
        }
      }
    }

    // 已登录用户访问登录/注册页，重定向到首页
    if ((to.name === 'Login' || to.name === 'Register') && isAuthenticated()) {
      return { name: 'Home' }
    }

    // 继续导航
    return true
  })

  // 监听 Token 过期事件
  window.addEventListener(TOKEN_EXPIRED_EVENT, () => {
    const authStore = useAuthStore()
    authStore.handleTokenExpired()
    router.push({
      name: 'Login',
      query: { redirect: router.currentRoute.value.fullPath },
    })
  })
}
