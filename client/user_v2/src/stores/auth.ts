import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/api/auth'
import type { User, LoginCredentials, RegisterData, AuthData } from '@/types/user'
import { STORAGE_KEYS } from '@/constants'

export const useAuthStore = defineStore(
  'auth',
  () => {
    const user = ref<User | null>(null)
    const accessToken = ref<string | null>(null)
    const refreshTokenVal = ref<string | null>(null)
    const error = ref<string | null>(null)

    const isAuthenticated = computed(() => !!accessToken.value)

    function applyAuthData(data: AuthData) {
      user.value = data.user
      accessToken.value = data.access_token
      refreshTokenVal.value = data.refresh_token
    }

    async function login(credentials: LoginCredentials) {
      error.value = null
      const res = await authApi.login(credentials)
      if (res.data) {
        user.value = res.data.user
        accessToken.value = res.data.access_token
        refreshTokenVal.value = res.data.refresh_token
      } else if (res.message) {
        error.value = res.message
      }
      return res
    }

    async function loginSendCode(email: string) {
      error.value = null
      const res = await authApi.loginSendCode(email)
      return res
    }

    async function loginWithCode(email: string, code: string) {
      error.value = null
      const res = await authApi.loginWithCode(email, code)
      if (res.data) {
        applyAuthData(res.data)
      } else if (res.message) {
        error.value = res.message
      }
      return res
    }

    async function registerSendCode(email: string) {
      error.value = null
      const res = await authApi.registerSendCode(email)
      return res
    }

    async function register(data: RegisterData) {
      error.value = null
      const res = await authApi.register(data)
      if (res.data) {
        applyAuthData(res.data)
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

    async function logout() {
      localStorage.removeItem(STORAGE_KEYS.ACCESS_TOKEN)
      user.value = null
      accessToken.value = null
      refreshTokenVal.value = null
      try {
        await authApi.logout()
      } catch {
        // ignore
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
      error,
      isAuthenticated,
      login,
      loginSendCode,
      loginWithCode,
      registerSendCode,
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
