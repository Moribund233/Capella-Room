import { computed } from 'vue'
import { useRouter } from 'vue-router'
import type { LoginCredentials, RegisterData } from '@/types/user'
import { useAuthStore } from '@/stores/auth'
import { ROUTE_PATHS } from '@/constants'

export function useAuth() {
  const router = useRouter()
  const authStore = useAuthStore()

  const isAuthenticated = computed(() => authStore.isAuthenticated)
  const user = computed(() => authStore.user)

  async function login(credentials: LoginCredentials, redirect?: string) {
    await authStore.login(credentials)
    router.push(redirect || ROUTE_PATHS.CHAT)
  }

  async function register(data: RegisterData) {
    await authStore.register(data)
    router.push(ROUTE_PATHS.LOGIN)
  }

  async function logout() {
    authStore.logout()
    router.push(ROUTE_PATHS.LOGIN)
  }

  return {
    user,
    isAuthenticated,
    login,
    register,
    logout,
  }
}
