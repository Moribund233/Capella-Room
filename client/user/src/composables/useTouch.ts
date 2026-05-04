import { ref, onMounted, onUnmounted, type Ref } from 'vue'

interface TouchOptions {
  threshold?: number
  onSwipeLeft?: () => void
  onSwipeRight?: () => void
  onSwipeUp?: () => void
  onSwipeDown?: () => void
  onLongPress?: () => void
  longPressDelay?: number
}

interface TouchState {
  startX: number
  startY: number
  startTime: number
  isLongPress: boolean
  longPressTimer: ReturnType<typeof setTimeout> | null
}

/**
 * 触摸手势组合式函数
 * @param elementRef - 目标元素引用
 * @param options - 手势配置选项
 */
export function useTouch(
  elementRef: Ref<HTMLElement | null>,
  options: TouchOptions = {}
) {
  const {
    threshold = 50,
    onSwipeLeft,
    onSwipeRight,
    onSwipeUp,
    onSwipeDown,
    onLongPress,
    longPressDelay = 500,
  } = options

  const state: TouchState = {
    startX: 0,
    startY: 0,
    startTime: 0,
    isLongPress: false,
    longPressTimer: null,
  }

  const isTouching = ref(false)

  function onTouchStart(e: TouchEvent) {
    const touch = e.touches[0]
    if (!touch) return
    state.startX = touch.clientX
    state.startY = touch.clientY
    state.startTime = Date.now()
    state.isLongPress = false
    isTouching.value = true

    // 设置长按定时器
    if (onLongPress) {
      state.longPressTimer = setTimeout(() => {
        state.isLongPress = true
        onLongPress()
      }, longPressDelay)
    }
  }

  function onTouchMove(e: TouchEvent) {
    if (!isTouching.value) return

    // 如果移动距离超过阈值，取消长按
    if (state.longPressTimer) {
      const touch = e.touches[0]
      if (!touch) return
      const deltaX = Math.abs(touch.clientX - state.startX)
      const deltaY = Math.abs(touch.clientY - state.startY)

      if (deltaX > 10 || deltaY > 10) {
        clearTimeout(state.longPressTimer)
        state.longPressTimer = null
      }
    }
  }

  function onTouchEnd(e: TouchEvent) {
    if (!isTouching.value) return
    isTouching.value = false

    // 清除长按定时器
    if (state.longPressTimer) {
      clearTimeout(state.longPressTimer)
      state.longPressTimer = null
    }

    // 如果是长按，不处理滑动
    if (state.isLongPress) return

    const touch = e.changedTouches[0]
    if (!touch) return
    const deltaX = touch.clientX - state.startX
    const deltaY = touch.clientY - state.startY
    const deltaTime = Date.now() - state.startTime

    // 快速滑动检测
    const isQuickSwipe = deltaTime < 300
    const effectiveThreshold = isQuickSwipe ? threshold * 0.5 : threshold

    // 判断滑动方向
    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      // 水平滑动
      if (Math.abs(deltaX) > effectiveThreshold) {
        if (deltaX > 0 && onSwipeRight) {
          onSwipeRight()
        } else if (deltaX < 0 && onSwipeLeft) {
          onSwipeLeft()
        }
      }
    } else {
      // 垂直滑动
      if (Math.abs(deltaY) > effectiveThreshold) {
        if (deltaY > 0 && onSwipeDown) {
          onSwipeDown()
        } else if (deltaY < 0 && onSwipeUp) {
          onSwipeUp()
        }
      }
    }
  }

  function onTouchCancel() {
    isTouching.value = false
    if (state.longPressTimer) {
      clearTimeout(state.longPressTimer)
      state.longPressTimer = null
    }
  }

  onMounted(() => {
    const element = elementRef.value
    if (!element) return

    element.addEventListener('touchstart', onTouchStart, { passive: true })
    element.addEventListener('touchmove', onTouchMove, { passive: true })
    element.addEventListener('touchend', onTouchEnd)
    element.addEventListener('touchcancel', onTouchCancel)
  })

  onUnmounted(() => {
    const element = elementRef.value
    if (!element) return

    element.removeEventListener('touchstart', onTouchStart)
    element.removeEventListener('touchmove', onTouchMove)
    element.removeEventListener('touchend', onTouchEnd)
    element.removeEventListener('touchcancel', onTouchCancel)

    if (state.longPressTimer) {
      clearTimeout(state.longPressTimer)
    }
  })

  return {
    isTouching,
  }
}

/**
 * 滑动手势组合式函数（简化版）
 * @param elementRef - 目标元素引用
 * @param onSwipe - 滑动回调函数
 */
export function useSwipe(
  elementRef: Ref<HTMLElement | null>,
  onSwipe: (direction: 'left' | 'right' | 'up' | 'down') => void
) {
  return useTouch(elementRef, {
    onSwipeLeft: () => onSwipe('left'),
    onSwipeRight: () => onSwipe('right'),
    onSwipeUp: () => onSwipe('up'),
    onSwipeDown: () => onSwipe('down'),
  })
}

/**
 * 长按手势组合式函数（简化版）
 * @param elementRef - 目标元素引用
 * @param onLongPress - 长按回调函数
 * @param delay - 长按延迟时间（毫秒）
 */
export function useLongPress(
  elementRef: Ref<HTMLElement | null>,
  onLongPress: () => void,
  delay = 500
) {
  return useTouch(elementRef, {
    onLongPress,
    longPressDelay: delay,
  })
}

/**
 * 双击手势组合式函数
 * @param elementRef - 目标元素引用
 * @param onDoubleTap - 双击回调函数
 * @param delay - 双击间隔时间（毫秒）
 */
export function useDoubleTap(
  elementRef: Ref<HTMLElement | null>,
  onDoubleTap: () => void,
  delay = 300
) {
  let lastTapTime = 0
  let tapCount = 0
  let tapTimer: ReturnType<typeof setTimeout> | null = null

  function onTouchEnd() {
    const currentTime = Date.now()
    const timeDiff = currentTime - lastTapTime

    if (timeDiff < delay) {
      tapCount++
      if (tapCount === 2) {
        onDoubleTap()
        tapCount = 0
        if (tapTimer) {
          clearTimeout(tapTimer)
          tapTimer = null
        }
      }
    } else {
      tapCount = 1
      if (tapTimer) {
        clearTimeout(tapTimer)
      }
      tapTimer = setTimeout(() => {
        tapCount = 0
      }, delay)
    }

    lastTapTime = currentTime
  }

  onMounted(() => {
    const element = elementRef.value
    if (!element) return
    element.addEventListener('touchend', onTouchEnd)
  })

  onUnmounted(() => {
    const element = elementRef.value
    if (!element) return
    element.removeEventListener('touchend', onTouchEnd)
    if (tapTimer) {
      clearTimeout(tapTimer)
    }
  })
}

/**
 * 捏合缩放手势组合式函数
 * @param elementRef - 目标元素引用
 * @param onPinch - 捏合回调函数
 */
export function usePinch(
  elementRef: Ref<HTMLElement | null>,
  onPinch: (scale: number) => void
) {
  let initialDistance = 0
  let currentScale = 1

  function getDistance(touches: TouchList) {
    const t0 = touches[0]
    const t1 = touches[1]
    if (!t0 || !t1) return 0
    const dx = t0.clientX - t1.clientX
    const dy = t0.clientY - t1.clientY
    return Math.sqrt(dx * dx + dy * dy)
  }

  function onTouchStart(e: TouchEvent) {
    if (e.touches.length === 2) {
      initialDistance = getDistance(e.touches)
    }
  }

  function onTouchMove(e: TouchEvent) {
    if (e.touches.length === 2 && initialDistance > 0) {
      const currentDistance = getDistance(e.touches)
      const scale = currentDistance / initialDistance
      currentScale = scale
      onPinch(scale)
    }
  }

  function onTouchEnd() {
    if (currentScale !== 1) {
      currentScale = 1
      initialDistance = 0
    }
  }

  onMounted(() => {
    const element = elementRef.value
    if (!element) return

    element.addEventListener('touchstart', onTouchStart, { passive: true })
    element.addEventListener('touchmove', onTouchMove, { passive: true })
    element.addEventListener('touchend', onTouchEnd)
  })

  onUnmounted(() => {
    const element = elementRef.value
    if (!element) return

    element.removeEventListener('touchstart', onTouchStart)
    element.removeEventListener('touchmove', onTouchMove)
    element.removeEventListener('touchend', onTouchEnd)
  })
}
