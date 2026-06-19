import { ref, onMounted, onUnmounted, type Ref } from 'vue'

interface SwipeOptions {
  onSwipeLeft?: () => void
  onSwipeRight?: () => void
  onSwipeUp?: () => void
  onSwipeDown?: () => void
  threshold?: number
  edgeThreshold?: number
}

export function useSwipeGesture(element: Ref<HTMLElement | null>, options: SwipeOptions = {}) {
  const {
    onSwipeLeft,
    onSwipeRight,
    onSwipeUp,
    onSwipeDown,
    threshold = 50,
    edgeThreshold = 30,
  } = options

  const startX = ref(0)
  const startY = ref(0)
  const currentX = ref(0)
  const currentY = ref(0)
  const isDragging = ref(false)
  const direction = ref<'horizontal' | 'vertical' | null>(null)

  function handleTouchStart(e: TouchEvent) {
    const touch = e.touches[0]
    if (!touch) return
    startX.value = touch.clientX
    startY.value = touch.clientY
    currentX.value = touch.clientX
    currentY.value = touch.clientY
    isDragging.value = true
    direction.value = null
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isDragging.value) return

    const touch = e.touches[0]
    if (!touch) return
    currentX.value = touch.clientX
    currentY.value = touch.clientY

    const deltaX = currentX.value - startX.value
    const deltaY = currentY.value - startY.value

    // Determine direction on first significant movement
    if (!direction.value) {
      if (Math.abs(deltaX) > 10 || Math.abs(deltaY) > 10) {
        direction.value = Math.abs(deltaX) > Math.abs(deltaY) ? 'horizontal' : 'vertical'
      }
    }

    // Prevent default scrolling for horizontal swipes
    if (direction.value === 'horizontal') {
      e.preventDefault()
    }
  }

  function handleTouchEnd() {
    if (!isDragging.value) return

    const deltaX = currentX.value - startX.value
    const deltaY = currentY.value - startY.value

    // Check if swipe started from edge (for opening sidebar)
    const isFromLeftEdge = startX.value <= edgeThreshold
    const isFromRightEdge = startX.value >= window.innerWidth - edgeThreshold

    // Handle horizontal swipes
    if (direction.value === 'horizontal') {
      if (deltaX > threshold && (isFromLeftEdge || deltaX > 0)) {
        onSwipeRight?.()
      } else if (deltaX < -threshold && (isFromRightEdge || deltaX < 0)) {
        onSwipeLeft?.()
      }
    }

    // Handle vertical swipes
    if (direction.value === 'vertical') {
      if (deltaY > threshold) {
        onSwipeDown?.()
      } else if (deltaY < -threshold) {
        onSwipeUp?.()
      }
    }

    isDragging.value = false
    direction.value = null
  }

  onMounted(() => {
    const el = element.value
    if (el) {
      el.addEventListener('touchstart', handleTouchStart, { passive: true })
      el.addEventListener('touchmove', handleTouchMove, { passive: false })
      el.addEventListener('touchend', handleTouchEnd, { passive: true })
    }
  })

  onUnmounted(() => {
    const el = element.value
    if (el) {
      el.removeEventListener('touchstart', handleTouchStart)
      el.removeEventListener('touchmove', handleTouchMove)
      el.removeEventListener('touchend', handleTouchEnd)
    }
  })

  return {
    isDragging,
    direction,
  }
}
