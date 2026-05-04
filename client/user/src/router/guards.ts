import type { Router } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const publicRoutes = ['login', 'register']

export function setupRouterGuards(router: Router) {
  router.beforeEach(async (to, _from) => {
    const authStore = useAuthStore()
    const isPublic = publicRoutes.includes(to.name as string)
    const token = authStore.accessToken

    if (!isPublic && !token) {
      return { name: 'login', query: { redirect: to.fullPath } }
    }

    if (isPublic && token && to.name === 'login') {
      return { name: 'chat' }
    }
  })
}
