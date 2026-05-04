import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/api/auth'
import type { User, LoginCredentials, RegisterData } from '@/types/user'
import { STORAGE_KEYS } from '@/constants'

export const useAuthStore = defineStore(
  'auth',
  () => {
    const user = ref<User | null>(null)
    const accessToken = ref<string | null>(null)
    const refreshTokenVal = ref<string | null>(null)

    const isAuthenticated = computed(() => !!accessToken.value)

    async function login(credentials: LoginCredentials) {
      const res = await authApi.login(credentials)
      if (res.data) {
        user.value = res.data.user
        accessToken.value = res.data.access_token
        refreshTokenVal.value = res.data.refresh_token
      }
      return res
    }

    async function register(data: RegisterData) {
      const res = await authApi.register(data)
      if (res.data) {
        user.value = res.data
      }
      return res
    }

    async function fetchUser() {
      try {
        const res = await authApi.getMe()
        if (res.data) {
          user.value = res.data
        }
        return res
      } catch {
        await logout()
        throw new Error('获取用户信息失败')
      }
    }

    async function refreshAccessToken(): Promise<boolean> {
      if (!refreshTokenVal.value) return false
      try {
        const res = await authApi.refresh(refreshTokenVal.value)
        if (res.data) {
          accessToken.value = res.data.access_token
          refreshTokenVal.value = res.data.refresh_token
          return true
        }
        return false
      } catch {
        await logout()
        return false
      }
    }

    /**
     * 登出
     * 先清除 localStorage，再清除状态，避免状态不一致
     */
    async function logout() {
      // 先清除 localStorage，确保持久化数据被删除
      localStorage.removeItem(STORAGE_KEYS.ACCESS_TOKEN)

      // 清除状态
      user.value = null
      accessToken.value = null
      refreshTokenVal.value = null

      // 调用后端登出接口（不等待结果）
      try {
        await authApi.logout()
      } catch {
        // 忽略后端错误
      }
    }

    function $reset() {
      user.value = null
      accessToken.value = null
      refreshTokenVal.value = null
    }

    return {
      user,
      accessToken,
      refreshToken: refreshTokenVal,
      isAuthenticated,
      login,
      register,
      fetchUser,
      refreshAccessToken,
      logout,
      $reset,
    }
  },
  {
    persist: {
      key: STORAGE_KEYS.ACCESS_TOKEN,
      pick: ['accessToken', 'refreshToken', 'user'],
      storage: localStorage,
    },
  },
)
