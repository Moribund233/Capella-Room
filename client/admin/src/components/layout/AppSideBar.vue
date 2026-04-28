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
            :class="{ 'is-active': activeKey === item.key }">
            <a class="nav-link" :href="item.path" @click.prevent="handleMenuClick(item)">
              <component :is="item.icon" class="nav-icon" :size="isCollapsed && isDesktop ? 20 : 18" />
              <span v-if="showMenuText" class="nav-text">{{ item.label }}</span>
              <ChevronRight v-if="showMenuText && item.children" class="nav-arrow" :size="14"
                :class="{ 'is-expanded': expandedKeys.includes(item.key) }" />
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
        <button class="footer-btn" @click="handleLogout">
          <LogOut class="footer-icon" :size="16" />
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
 * 支持子路由匹配：当前路由以菜单项 path 开头即视为匹配
 */
watch(
  () => route.path,
  (newPath) => {
    // 优先精确匹配
    const exactMatch = menuItems.value.find((item) => item.path === newPath)
    if (exactMatch) {
      activeKey.value = exactMatch.key
      return
    }

    // 子路由匹配：找到最长匹配的父路径
    const matched = menuItems.value
      .filter((item) => newPath.startsWith(item.path) && item.path !== '/')
      .sort((a, b) => b.path.length - a.path.length)[0]

    if (matched) {
      activeKey.value = matched.key
    }
  },
  { immediate: true },
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
  dialog.warning({
    title: '确认退出',
    content: '确定要退出登录吗？',
    positiveText: '确认退出',
    negativeText: '取消',
    onPositiveClick: async () => {
      // 移动端关闭菜单
      if (isMobile.value) {
        emit('update:isMobileMenuOpen', false)
      }

      // 清除登录状态
      authStore.logout()

      // 跳转到登录页（使用 replace 避免返回按钮回到需要登录的页面）
      await router.replace({ name: 'Login' })
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
  transition: var(--transition-base);
  z-index: 99;
  display: flex;
  flex-direction: column;
  border-radius: 0 var(--sidebar-border-radius) var(--sidebar-border-radius) 0;
  opacity: var(--sidebar-opacity);
  border: var(--layout-border-width) var(--layout-border-style) var(--layout-border-color);
  border-left: none;
}

/* 移动端样式 */
.app-sidebar.is-mobile-open {
  z-index: 9999;
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
    border: none;
    border-radius: 0;
  }

  .app-sidebar.is-mobile-open {
    pointer-events: auto;
  }

  .sidebar-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
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
    transition: transform 0.3s cubic-bezier(0.645, 0.045, 0.355, 1);
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    padding-top: 0;
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
  }
}

/* 导航菜单 */
.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 12px 0;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  margin: 4px 0;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  color: var(--sidebar-text);
  text-decoration: none;
  cursor: pointer;
  transition: var(--transition-fast);
  position: relative;
}

.nav-link:hover {
  color: var(--sidebar-text-active);
  background: var(--sidebar-bg-hover);
}

.nav-item.is-active>.nav-link {
  color: var(--sidebar-text-active);
  background: var(--sidebar-bg-active);
}

.nav-icon {
  flex-shrink: 0;
  transition: var(--transition-fast);
}

.nav-text {
  flex: 1;
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: opacity 0.2s;
}

.nav-arrow {
  flex-shrink: 0;
  transition: transform 0.3s;
}

.nav-arrow.is-expanded {
  transform: rotate(90deg);
}

/* 折叠状态下的图标居中 */
.app-sidebar.is-collapsed .nav-link {
  justify-content: center;
  padding: 16px 0;
}

.app-sidebar.is-collapsed .nav-icon {
  margin: 0;
}

/* 子菜单 */
.submenu-list {
  list-style: none;
  padding: 0;
  margin: 0;
  background: rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.submenu-item {
  margin: 0;
}

.submenu-link {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px 10px 50px;
  color: var(--sidebar-text);
  text-decoration: none;
  cursor: pointer;
  transition: var(--transition-fast);
}

.submenu-link:hover {
  color: var(--sidebar-text-active);
}

.submenu-item.is-active .submenu-link {
  color: var(--sidebar-text-active);
}

.submenu-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.5;
}

.submenu-item.is-active .submenu-dot {
  opacity: 1;
  background: var(--color-primary);
}

.submenu-text {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 子菜单动画 */
.submenu-enter-active,
.submenu-leave-active {
  transition: all 0.3s ease;
  max-height: 200px;
}

.submenu-enter-from,
.submenu-leave-to {
  max-height: 0;
  opacity: 0;
}

/* 侧边栏底部 */
.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.footer-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 16px;
  border: none;
  background: transparent;
  color: var(--sidebar-text);
  font-size: 14px;
  cursor: pointer;
  border-radius: 6px;
  transition: var(--transition-fast);
}

.footer-btn:hover {
  background: var(--sidebar-bg-hover);
  color: var(--sidebar-text-active);
}

.footer-icon {
  flex-shrink: 0;
}

/* 移动端适配 */
@media screen and (max-width: 767px) {
  .sidebar-nav {
    padding: 8px 0;
  }

  .nav-link {
    padding: 14px 20px;
  }

  .submenu-link {
    padding: 12px 20px 12px 50px;
  }
}

/* 平板端适配 */
@media screen and (min-width: 768px) and (max-width: 1024px) {
  .app-sidebar {
    top: var(--header-height);
  }
}
</style>
