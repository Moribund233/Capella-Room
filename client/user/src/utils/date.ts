const MINUTE = 60
const HOUR = 3600
const DAY = 86400

function padZero(n: number): string {
  return n.toString().padStart(2, '0')
}

export function formatTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = Math.floor((now.getTime() - date.getTime()) / 1000)

  if (diff < MINUTE) return '刚刚'
  if (diff < HOUR) return `${Math.floor(diff / MINUTE)} 分钟前`

  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const dateDay = new Date(date.getFullYear(), date.getMonth(), date.getDate())
  const dayDiff = Math.floor((today.getTime() - dateDay.getTime()) / DAY)

  const hours = padZero(date.getHours())
  const minutes = padZero(date.getMinutes())

  if (dayDiff === 0) return `${hours}:${minutes}`
  if (dayDiff === 1) return `昨天 ${hours}:${minutes}`
  if (dayDiff < 7) return `${dayDiff} 天前`
  if (date.getFullYear() === now.getFullYear()) {
    return `${padZero(date.getMonth() + 1)}/${padZero(date.getDate())} ${hours}:${minutes}`
  }
  return `${date.getFullYear()}/${padZero(date.getMonth() + 1)}/${padZero(date.getDate())}`
}

export function formatDate(dateStr: string | undefined | null): string {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return `${date.getFullYear()}-${padZero(date.getMonth() + 1)}-${padZero(date.getDate())}`
}

export function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr)
  return `${formatDate(dateStr)} ${padZero(date.getHours())}:${padZero(date.getMinutes())}`
}

export function isSameDay(a: string, b: string): boolean {
  const da = new Date(a)
  const db = new Date(b)
  return (
    da.getFullYear() === db.getFullYear() &&
    da.getMonth() === db.getMonth() &&
    da.getDate() === db.getDate()
  )
}

export function shouldShowTimeSeparator(
  current: string,
  previous: string | null,
): boolean {
  if (!previous) return true
  return !isSameDay(current, previous)
}
