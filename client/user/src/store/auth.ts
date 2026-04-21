import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserInfo } from '@/types'

/**
 * Token 存储键名
 */
const TOKEN_KEY = 'auth_token'

/**
 * 认证状态存储
 */
export const useAuthStore = defineStore('auth', () => {
  /**
   * Token
   */
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))

  /**
   * 用户信息
   */
  const userInfo = ref<UserInfo | null>(null)

  /**
   * 是否已登录
   */
  const isLoggedIn = computed(() => !!token.value)

  /**
   * 设置 Token
   * @param newToken 新的 Token
   */
  function setToken(newToken: string) {
    token.value = newToken
    localStorage.setItem(TOKEN_KEY, newToken)
  }

  /**
   * 清除 Token
   */
  function clearToken() {
    token.value = null
    userInfo.value = null
    localStorage.removeItem(TOKEN_KEY)
  }

  /**
   * 设置用户信息
   * @param info 用户信息
   */
  function setUserInfo(info: UserInfo) {
    userInfo.value = info
  }

  /**
   * 退出登录
   */
  function logout() {
    clearToken()
  }

  return {
    token,
    userInfo,
    isLoggedIn,
    setToken,
    clearToken,
    setUserInfo,
    logout,
  }
})
