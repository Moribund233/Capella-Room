import axios, { type AxiosRequestConfig, type AxiosError } from 'axios'
import type { ApiResponse } from '@/types/api'
import { STORAGE_KEYS } from '@/constants'

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

httpClient.interceptors.response.use(
  (response) => response.data,
  async (error: AxiosError<ApiResponse<unknown>>) => {
    const originalRequest = error.config as AxiosRequestConfig & { _retry?: boolean }

    if (error.response?.status === 401 && !originalRequest._retry) {
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
        }

        processQueue(null, newAccessToken)
        return httpClient(originalRequest)
      } catch (refreshError) {
        processQueue(refreshError, null)
        clearPersistedTokens()
        window.location.href = '/login'
        return Promise.reject(refreshError)
      } finally {
        isRefreshing = false
      }
    }

    return Promise.reject(error)
  },
)

export default httpClient
