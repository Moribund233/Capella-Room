const HEALTH_ENDPOINT = '/health'
const CHECK_TIMEOUT = 5000

export type ServerStatus = 'reachable' | 'unreachable'

/**
 * 轻量级服务器存活检查。
 * 超时 5s，不依赖 axios（防止拦截器层面的干扰）。
 */
export async function checkServerHealth(): Promise<ServerStatus> {
  const baseUrl = import.meta.env.VITE_API_BASE_URL
  if (!baseUrl) return 'unreachable'

  try {
    const origin = new URL(baseUrl).origin
    const url = `${origin}${HEALTH_ENDPOINT}`

    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), CHECK_TIMEOUT)

    const res = await fetch(url, { signal: controller.signal })
    clearTimeout(timer)

    if (!res.ok) return 'unreachable'

    const body = await res.json()
    return body?.data?.status === 'healthy' ? 'reachable' : 'unreachable'
  } catch {
    return 'unreachable'
  }
}
