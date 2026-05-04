import { ref, computed, onMounted, onUnmounted } from 'vue'

const breakpoints = {
  xs: 0,
  s: 640,
  m: 768,
  l: 1024,
  xl: 1280,
  xxl: 1536,
}

const windowWidth = ref(window.innerWidth)

function onResize() {
  windowWidth.value = window.innerWidth
}

export function useResponsive() {
  onMounted(() => window.addEventListener('resize', onResize))
  onUnmounted(() => window.removeEventListener('resize', onResize))

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
    sidebarCollapsed,
    toggleSidebar,
  }
}
