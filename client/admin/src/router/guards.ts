import type { Router, RouteLocationNormalized } from 'vue-router'
import { useAuthStore } from '@/store'

/**
 * 设置路由守卫
 * @param router 路由实例
 */
export function setupRouterGuards(router: Router): void {
  router.beforeEach((to: RouteLocationNormalized) => {
    const authStore = useAuthStore()
    const requiresAuth = to.meta.requiresAuth as boolean

    if (requiresAuth && !authStore.isLoggedIn) {
      return { name: 'Login', query: { redirect: to.fullPath } }
    }

    if (to.name === 'Login' && authStore.isLoggedIn) {
      return { name: 'Home' }
    }

    return true
  })
}
