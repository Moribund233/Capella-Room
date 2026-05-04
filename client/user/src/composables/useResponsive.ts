import { ref, computed, onMounted, onUnmounted } from 'vue'

const breakpoints = {
  xs: 0,
  s: 640,
  m: 768,
  l: 1024,
  xl: 1280,
  xxl: 1536,
}

// 使用全局状态确保所有组件共享同一个响应式数据
const windowWidth = ref(0)
let resizeListener: (() => void) | null = null
let listenerCount = 0

function initWindowWidth() {
  if (typeof window !== 'undefined') {
    windowWidth.value = window.innerWidth
  }
}

function addResizeListener() {
  if (resizeListener || typeof window === 'undefined') return

  resizeListener = () => {
    windowWidth.value = window.innerWidth
  }
  window.addEventListener('resize', resizeListener)
}

function removeResizeListener() {
  if (!resizeListener || listenerCount > 0) return
  window.removeEventListener('resize', resizeListener)
  resizeListener = null
}

export function useResponsive() {
  // 初始化窗口宽度
  onMounted(() => {
    if (windowWidth.value === 0) {
      initWindowWidth()
    }
    listenerCount++
    addResizeListener()
  })

  onUnmounted(() => {
    listenerCount--
    removeResizeListener()
  })

  const breakpoint = computed(() => {
    const w = windowWidth.value
    if (w < breakpoints.s) return 'xs'
    if (w < breakpoints.m) return 's'
    if (w < breakpoints.l) return 'm'
    if (w < breakpoints.xl) return 'l'
    if (w < breakpoints.xxl) return 'xl'
    return 'xxl'
  })

  const isMobile = computed(() => windowWidth.value < breakpoints.m)
  const isTablet = computed(() => windowWidth.value >= breakpoints.m && windowWidth.value < breakpoints.l)
  const isDesktop = computed(() => windowWidth.value >= breakpoints.l)

  // 布局模式：根据设备类型确定使用哪种布局
  const layoutMode = computed(() => {
    if (windowWidth.value < breakpoints.m) return 'mobile'
    if (windowWidth.value < breakpoints.l) return 'tablet'
    return 'desktop'
  })

  const sidebarCollapsed = ref(false)

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  return {
    windowWidth,
    breakpoint,
    isMobile,
    isTablet,
    isDesktop,
    layoutMode,
    sidebarCollapsed,
    toggleSidebar,
  }
}
