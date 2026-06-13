import axios, { type AxiosRequestConfig, type AxiosError } from 'axios'
import type { ApiResponse } from '@/types/api'
import { STORAGE_KEYS } from '@/constants'
import { useAuthStore } from '@/stores/auth'
import router from '@/router'

function readPersistedToken(key: string): string | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS.ACCESS_TOKEN)
    if (!raw) return null
    const parsed = JSON.parse(raw)
    return parsed[key] ?? null
  } catch {
    return null
  }
}

function writePersistedToken(key: string, value: string) {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS.ACCESS_TOKEN)
    const parsed = raw ? JSON.parse(raw) : {}
    parsed[key] = value
    localStorage.setItem(STORAGE_KEYS.ACCESS_TOKEN, JSON.stringify(parsed))
  } catch {
    // ignore write errors
  }
}

function clearPersistedTokens() {
  localStorage.removeItem(STORAGE_KEYS.ACCESS_TOKEN)
}

const httpClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

httpClient.interceptors.request.use(
  (config) => {
    const token = readPersistedToken('accessToken')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => Promise.reject(error),
)

let isRefreshing = false
let pendingQueue: Array<{
  resolve: (value: unknown) => void
  reject: (reason: unknown) => void
}> = []

function processQueue(error: unknown, token: string | null = null) {
  pendingQueue.forEach((prom) => {
    if (error) {
      prom.reject(error)
    } else {
      prom.resolve(token)
    }
  })
  pendingQueue = []
}

/**
 * 处理 401 未授权错误
 * 清除认证状态并重定向到登录页
 */
async function handleUnauthorized() {
  // 清除 localStorage
  clearPersistedTokens()

  // 获取 auth store 并登出
  try {
    const authStore = useAuthStore()
    // 重置状态（不调用 logout API，因为 token 已经无效）
    authStore.$reset()
  } catch {
    // 如果 store 未初始化，忽略错误
  }

  // 重定向到登录页（如果不是已经在登录页）
  if (router.currentRoute.value.name !== 'login') {
    router.push({ name: 'login', query: { redirect: router.currentRoute.value.fullPath } })
  }
}

httpClient.interceptors.response.use(
  (response) => response.data,
  async (error: AxiosError<ApiResponse<unknown>>) => {
    const originalRequest = error.config as AxiosRequestConfig & { _retry?: boolean }

    // 处理 401 错误
    if (error.response?.status === 401) {
      // 登录/注册接口的 401 是业务错误（密码错误/用户不存在），直接透传
      if (originalRequest.url?.includes('/auth/login') ||
          originalRequest.url?.includes('/auth/register')) {
        return Promise.reject(error)
      }

      // 如果是刷新 token 的请求失败，直接处理未授权
      if (originalRequest.url?.includes('/auth/refresh')) {
        await handleUnauthorized()
        return Promise.reject(error)
      }

      // 如果不是重试请求，尝试刷新 token
      if (!originalRequest._retry) {
        if (isRefreshing) {
          return new Promise((resolve, reject) => {
            pendingQueue.push({ resolve, reject })
          }).then(() => httpClient(originalRequest))
        }

        originalRequest._retry = true
        isRefreshing = true

        try {
          const refreshToken = readPersistedToken('refreshToken')
          if (!refreshToken) throw new Error('No refresh token')

          const { data } = await axios.post<
            ApiResponse<{ access_token: string; refresh_token: string }>
          >(
            `${import.meta.env.VITE_API_BASE_URL}/auth/refresh`,
            { refresh_token: refreshToken },
          )

          const newAccessToken = data.data?.access_token
          const newRefreshToken = data.data?.refresh_token
          if (newAccessToken) {
            writePersistedToken('accessToken', newAccessToken)
            writePersistedToken('refreshToken', newRefreshToken ?? refreshToken)

            // 更新 Pinia store 中的 token
            try {
              const authStore = useAuthStore()
              authStore.accessToken = newAccessToken
              if (newRefreshToken) {
                authStore.refreshToken = newRefreshToken
              }
            } catch {
              // 如果 store 未初始化，忽略错误
            }
          }

          processQueue(null, newAccessToken)
          return httpClient(originalRequest)
        } catch (refreshError) {
          processQueue(refreshError, null)
          await handleUnauthorized()
          return Promise.reject(refreshError)
        } finally {
          isRefreshing = false
        }
      }

      // 已经是重试请求且仍然 401，处理未授权
      await handleUnauthorized()
    }

    return Promise.reject(error)
  },
)

export default httpClient
