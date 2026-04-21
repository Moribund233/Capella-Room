/**
 * 响应式布局 Composable
 * 提供移动端检测和断点响应
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'

// 断点定义
const BREAKPOINTS = {
  mobile: 768,
  tablet: 1024,
  desktop: 1280,
}

/**
 * 响应式布局
 */
export function useResponsive() {
  const windowWidth = ref(window.innerWidth)
  const windowHeight = ref(window.innerHeight)

  // 是否移动端
  const isMobile = computed(() => windowWidth.value < BREAKPOINTS.mobile)

  // 是否平板
  const isTablet = computed(() =>
    windowWidth.value >= BREAKPOINTS.mobile && windowWidth.value < BREAKPOINTS.tablet
  )

  // 是否桌面端
  const isDesktop = computed(() => windowWidth.value >= BREAKPOINTS.tablet)

  // 是否大屏幕
  const isLargeScreen = computed(() => windowWidth.value >= BREAKPOINTS.desktop)

  // 更新窗口尺寸
  function updateWindowSize() {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  }

  onMounted(() => {
    window.addEventListener('resize', updateWindowSize)
  })

  onUnmounted(() => {
    window.removeEventListener('resize', updateWindowSize)
  })

  return {
    windowWidth,
    windowHeight,
    isMobile,
    isTablet,
    isDesktop,
    isLargeScreen,
    BREAKPOINTS,
  }
}

export default useResponsive
