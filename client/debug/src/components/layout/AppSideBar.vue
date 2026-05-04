<template>
  <aside class="app-sidebar" :class="{
    'is-collapsed': isCollapsed && isDesktop,
    'is-mobile-open': isMobileMenuOpen && isMobile,
    'is-tablet': isTablet,
  }" :style="sidebarStyle">
    <!-- 移动端遮罩层 -->
    <div v-if="isMobile && isMobileMenuOpen" class="sidebar-overlay" @click="closeMobileMenu"></div>

    <!-- 侧边栏内容 -->
    <div class="sidebar-content">
      <!-- 导航菜单 -->
      <nav class="sidebar-nav">
        <ul class="nav-list">
          <li v-for="item in menuItems" :key="item.key" class="nav-item"
            :class="{ 'is-active': activeKey === item.key, 'is-expanded': expandedKeys.includes(item.key) }">
            <a class="nav-link" :href="item.path" @click.prevent="handleMenuClick(item)">
              <div class="nav-icon-wrapper">
                <component :is="item.icon" class="nav-icon" :size="isCollapsed && isDesktop ? 22 : 20" />
              </div>
              <span v-if="showMenuText" class="nav-text">{{ item.label }}</span>
              <div v-if="showMenuText && item.children" class="nav-arrow-wrapper">
                <ChevronRight class="nav-arrow" :size="16" />
              </div>
              <!-- 激活指示器 -->
              <div class="active-indicator"></div>
            </a>

            <!-- 子菜单 -->
            <transition name="submenu">
              <ul v-if="item.children && expandedKeys.includes(item.key) && showMenuText" class="submenu-list">
                <li v-for="child in item.children" :key="child.key" class="submenu-item"
                  :class="{ 'is-active': activeKey === child.key }">
                  <a class="submenu-link" :href="child.path" @click.prevent="handleMenuClick(child)">
                    <span class="submenu-dot"></span>
                    <span class="submenu-text">{{ child.label }}</span>
                  </a>
                </li>
              </ul>
            </transition>
          </li>
        </ul>
      </nav>

      <!-- 底部操作区 -->
      <div v-if="showMenuText" class="sidebar-footer">
        <div class="footer-divider"></div>
        <button class="footer-btn" @click="handleLogout">
          <div class="footer-icon-wrapper">
            <LogOut class="footer-icon" :size="18" />
          </div>
          <span>退出登录</span>
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { LogOut, ChevronRight } from 'lucide-vue-next'
import { useSidebarConfig, type MenuItem } from '@/composables'
import { useAuthStore } from '@/store'
import { useDialog } from 'naive-ui'

/**
 * 组件属性定义
 */
interface Props {
  /** 是否折叠（桌面端） */
  isCollapsed?: boolean
  /** 移动端菜单是否打开 */
  isMobileMenuOpen?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isCollapsed: false,
  isMobileMenuOpen: false,
})

/**
 * 组件事件定义
 */
interface Emits {
  /** 更新移动端菜单状态 */
  (e: 'update:isMobileMenuOpen', value: boolean): void
}

const emit = defineEmits<Emits>()

/** 响应式断点 */
const isDesktop = ref(window.innerWidth > 1024)
const isTablet = ref(window.innerWidth >= 768 && window.innerWidth <= 1024)
const isMobile = ref(window.innerWidth < 768)

/** 当前激活的菜单项 */
const activeKey = ref('home')

/** 展开的子菜单项 */
const expandedKeys = ref<string[]>([])

/** 路由实例 */
const route = useRoute()
const router = useRouter()

/** 认证 store */
const authStore = useAuthStore()

/** 弹窗 */
const dialog = useDialog()

/**
 * 监听窗口大小变化
 */
const updateBreakpoint = () => {
  const width = window.innerWidth
  isDesktop.value = width > 1024
  isTablet.value = width >= 768 && width <= 1024
  isMobile.value = width < 768

  // 切换到桌面端时关闭移动端菜单
  if (!isMobile.value && props.isMobileMenuOpen) {
    emit('update:isMobileMenuOpen', false)
  }
}

window.addEventListener('resize', updateBreakpoint)

/**
 * 菜单配置 - 从 TOML 配置文件加载
 */
const { menuItems } = useSidebarConfig()

/**
 * 监听路由变化，更新激活菜单项
 * 支持子路由匹配：如果当前路径以菜单项路径开头，则视为匹配
 */
watch(
  () => route.path,
  (newPath) => {
    // 优先查找精确匹配
    const exactMatch = menuItems.value.find((item) => item.path === newPath)
    if (exactMatch) {
      activeKey.value = exactMatch.key
      return
    }

    // 查找子路由匹配（当前路径以菜单项路径开头，且菜单项路径不是根路径 '/'）
    const parentMatch = menuItems.value
      .filter((item) => item.path !== '/')
      .sort((a, b) => b.path.length - a.path.length) // 优先匹配更长的路径
      .find((item) => newPath.startsWith(item.path))

    if (parentMatch) {
      activeKey.value = parentMatch.key
    }
  },
  { immediate: true }
)

/**
 * 计算侧边栏样式
 */
const sidebarStyle = computed(() => {
  if (isMobile.value) {
    return {}
  }

  let width: string
  if (isDesktop.value) {
    width = props.isCollapsed ? 'var(--sidebar-width-collapsed)' : 'var(--sidebar-width-expanded)'
  } else {
    width = props.isCollapsed ? 'var(--sidebar-width-tablet-collapsed)' : 'var(--sidebar-width-tablet-expanded)'
  }

  return { width }
})

/**
 * 是否显示菜单文字
 */
const showMenuText = computed(() => {
  if (isMobile.value) return true
  if (isDesktop.value) return !props.isCollapsed
  return !props.isCollapsed
})

/**
 * 关闭移动端菜单
 */
const closeMobileMenu = () => {
  emit('update:isMobileMenuOpen', false)
}

/**
 * 处理菜单点击
 */
const handleMenuClick = (item: MenuItem) => {
  activeKey.value = item.key

  // 切换子菜单展开状态
  if (item.children) {
    const index = expandedKeys.value.indexOf(item.key)
    if (index > -1) {
      expandedKeys.value.splice(index, 1)
    } else {
      expandedKeys.value.push(item.key)
    }
  } else {
    // 路由跳转
    router.push(item.path)
  }

  // 移动端点击后关闭菜单
  if (isMobile.value) {
    emit('update:isMobileMenuOpen', false)
  }
}

/**
 * 处理退出登录
 */
const handleLogout = () => {
  // 提前捕获 router 实例和 emit 函数，避免在异步回调中调用
  const currentRouter = router
  const currentEmit = emit
  const currentIsMobile = isMobile.value

  dialog.warning({
    title: '确认退出',
    content: '确定要退出登录吗？',
    positiveText: '确认退出',
    negativeText: '取消',
    onPositiveClick: async () => {
      // 移动端关闭菜单
      if (currentIsMobile) {
        currentEmit('update:isMobileMenuOpen', false)
      }

      // 清除登录状态
      await authStore.logout()

      // 跳转到登录页（使用 replace 避免返回按钮回到需要登录的页面）
      await currentRouter.replace({ name: 'Login' })
    },
  })
}
</script>

<style scoped>
.app-sidebar {
  position: fixed;
  left: 0;
  top: var(--sidebar-top-offset, var(--header-height));
  height: var(--sidebar-compact-height, calc(100vh - var(--header-height) - var(--footer-height)));
  background: var(--sidebar-bg);
  transition: all var(--duration-slow) var(--ease-out-expo);
  z-index: 99;
  display: flex;
  flex-direction: column;
  border-radius: 0 var(--sidebar-border-radius) var(--sidebar-border-radius) 0;
  opacity: var(--sidebar-opacity);
  border: var(--layout-border-width) var(--layout-border-style) var(--layout-border-color);
  border-left: none;
}

/* 移动端遮罩层 */
.sidebar-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  z-index: -1;
}

/* 移动端样式 */
.app-sidebar.is-mobile-open {
  z-index: 100;
}

.app-sidebar.is-mobile-open .sidebar-content {
  transform: translateX(0);
}

/* 移动端侧边栏内容 */
@media screen and (max-width: 767px) {
  .app-sidebar {
    top: var(--header-height-mobile);
    width: 100%;
    background: transparent;
    pointer-events: none;
    z-index: 100;
  }

  .app-sidebar.is-mobile-open {
    pointer-events: auto;
  }

  .sidebar-content {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 40vw;
    min-width: 240px;
    max-width: 320px;
    height: calc(100vh - var(--header-height-mobile));
    background: var(--sidebar-bg);
    transform: translateX(-100%);
    transition: transform var(--duration-slow) var(--ease-out-expo);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    padding-top: 0;
    border-radius: 0 var(--radius-xl) var(--radius-xl) 0;
    box-shadow: var(--shadow-xl);
  }
}

/* 桌面端和平板端样式 */
@media screen and (min-width: 768px) {
  .sidebar-content {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: var(--space-4) 0;
  }
}

/* 导航菜单 */
.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0 var(--space-3);
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.nav-item {
  position: relative;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  color: var(--sidebar-text);
  text-decoration: none;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-smooth);
  position: relative;
  border-radius: var(--radius-lg);
}

.nav-link:hover {
  color: var(--sidebar-text-active);
  background: var(--sidebar-bg-hover);
}

.nav-item.is-active > .nav-link {
  color: var(--sidebar-text-active);
  background: var(--sidebar-bg-active);
}

/* 激活指示器 */
.active-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%) scaleY(0);
  width: 3px;
  height: 20px;
  background: var(--color-primary-gradient);
  border-radius: 0 var(--radius-full) var(--radius-full) 0;
  transition: transform var(--duration-normal) var(--ease-spring);
}

.nav-item.is-active .active-indicator {
  transform: translateY(-50%) scaleY(1);
}

.nav-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}

.nav-icon {
  flex-shrink: 0;
  transition: all var(--duration-fast) var(--ease-smooth);
}

.nav-text {
  flex: 1;
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: opacity var(--duration-fast) var(--ease-smooth);
}

.nav-arrow-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform var(--duration-normal) var(--ease-smooth);
}

.nav-arrow {
  flex-shrink: 0;
  transition: transform var(--duration-normal) var(--ease-smooth);
}

.nav-item.is-expanded .nav-arrow {
  transform: rotate(90deg);
}

/* 折叠状态下的图标居中 */
.app-sidebar.is-collapsed .nav-link {
  justify-content: center;
  padding: var(--space-4) 0;
}

.app-sidebar.is-collapsed .nav-icon-wrapper {
  margin: 0;
}

.app-sidebar.is-collapsed .active-indicator {
  left: auto;
  right: 0;
  border-radius: var(--radius-full) 0 0 var(--radius-full);
}

/* 子菜单 */
.submenu-list {
  list-style: none;
  padding: var(--space-2) 0 var(--space-2) var(--space-4);
  margin: 0;
  overflow: hidden;
}

.submenu-item {
  margin: 0;
}

.submenu-link {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-4);
  color: var(--sidebar-text);
  text-decoration: none;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-smooth);
  border-radius: var(--radius-md);
  position: relative;
}

.submenu-link:hover {
  color: var(--sidebar-text-active);
  background: var(--sidebar-bg-hover);
}

.submenu-item.is-active .submenu-link {
  color: var(--sidebar-text-active);
  background: var(--sidebar-bg-active);
}

.submenu-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.5;
  transition: all var(--duration-fast) var(--ease-smooth);
}

.submenu-item.is-active .submenu-dot {
  opacity: 1;
  background: var(--color-primary);
  transform: scale(1.2);
}

.submenu-text {
  font-size: 13px;
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 子菜单动画 */
.submenu-enter-active,
.submenu-leave-active {
  transition: all var(--duration-normal) var(--ease-out-expo);
  max-height: 300px;
  opacity: 1;
}

.submenu-enter-from,
.submenu-leave-to {
  max-height: 0;
  opacity: 0;
}

/* 侧边栏底部 */
.sidebar-footer {
  padding: var(--space-4);
  margin-top: auto;
}

.footer-divider {
  height: 1px;
  background: var(--border-color-split);
  margin-bottom: var(--space-4);
}

.footer-btn {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: none;
  background: transparent;
  color: var(--sidebar-text);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border-radius: var(--radius-lg);
  transition: all var(--duration-fast) var(--ease-smooth);
}

.footer-btn:hover {
  background: var(--sidebar-bg-hover);
  color: var(--sidebar-text-active);
}

.footer-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
}

.footer-icon {
  flex-shrink: 0;
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .app-sidebar {
    top: var(--header-height-mobile);
    width: 100%;
    background: transparent;
    pointer-events: none;
    border: none;
  }

  .sidebar-nav {
    padding: var(--space-3);
  }

  .nav-link {
    padding: var(--space-4);
  }

  .submenu-link {
    padding: var(--space-3) var(--space-4);
  }

  .sidebar-footer {
    padding: var(--space-3) var(--space-4);
  }
}

/* 平板端适配 */
@media screen and (min-width: 768px) and (max-width: 1024px) {
  .nav-link {
    padding: var(--space-3);
  }
}
</style>
