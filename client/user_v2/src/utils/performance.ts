/**
 * 防抖函数
 * @param fn 要执行的函数
 * @param delay 延迟时间（毫秒）
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout> | null = null

  return function (this: unknown, ...args: Parameters<T>) {
    if (timer) {
      clearTimeout(timer)
    }
    timer = setTimeout(() => {
      fn.apply(this, args)
      timer = null
    }, delay)
  }
}

/**
 * 节流函数
 * @param fn 要执行的函数
 * @param limit 限制时间（毫秒）
 * @returns 节流后的函数
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false

  return function (this: unknown, ...args: Parameters<T>) {
    if (!inThrottle) {
      fn.apply(this, args)
      inThrottle = true
      setTimeout(() => {
        inThrottle = false
      }, limit)
    }
  }
}

/**
 * 请求动画帧节流
 * @param fn 要执行的函数
 * @returns 节流后的函数
 */
export function rafThrottle<T extends (...args: unknown[]) => unknown>(
  fn: T
): (...args: Parameters<T>) => void {
  let ticking = false

  return function (this: unknown, ...args: Parameters<T>) {
    if (!ticking) {
      window.requestAnimationFrame(() => {
        fn.apply(this, args)
        ticking = false
      })
      ticking = true
    }
  }
}

/**
 * 延迟执行
 * @param ms 延迟毫秒数
 * @returns Promise
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * 批量处理函数
 * @param items 要处理的数组
 * @param batchSize 每批处理数量
 * @param processor 处理函数
 * @param delay 批次间延迟（毫秒）
 */
export async function batchProcess<T>(
  items: T[],
  batchSize: number,
  processor: (item: T) => void | Promise<void>,
  delay = 0
): Promise<void> {
  for (let i = 0; i < items.length; i += batchSize) {
    const batch = items.slice(i, i + batchSize)
    await Promise.all(batch.map((item) => processor(item)))

    if (i + batchSize < items.length && delay > 0) {
      await sleep(delay)
    }
  }
}
