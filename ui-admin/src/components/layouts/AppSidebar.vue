<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import {
  HomeFilled,
  Setting,
  UserFilled,
  Monitor,
  Document,
  DocumentChecked,
} from '@element-plus/icons-vue'

/**
 * 侧边栏组件
 * 提供导航菜单和折叠功能
 */

interface MenuItem {
  path: string
  name: string
  icon: any
  children?: MenuItem[]
}

const route = useRoute()

/** 定义props */
const props = defineProps<{
  collapsed?: boolean
  mobileOpen?: boolean
}>()

/** 定义事件 */
const emit = defineEmits<{
  'update:collapsed': [value: boolean]
  'update:mobileOpen': [value: boolean]
}>()

/** 是否折叠 */
const isCollapsed = computed({
  get: () => props.collapsed ?? false,
  set: (value) => emit('update:collapsed', value),
})

/** 移动端菜单是否打开 */
const isMobileOpen = computed({
  get: () => props.mobileOpen ?? false,
  set: (value) => emit('update:mobileOpen', value),
})

/**
 * 关闭移动端侧边栏
 */
const closeMobileSidebar = () => {
  isMobileOpen.value = false
}



/** 菜单列表 */
const menuList = ref<MenuItem[]>([
  {
    path: '/dashboard',
    name: '仪表盘',
    icon: HomeFilled,
  },
  {
    path: '/users',
    name: '用户管理',
    icon: UserFilled,
  },
  {
    path: '/monitor',
    name: '系统监控',
    icon: Monitor,
  },
  {
    path: '/audit',
    name: '审计系统',
    icon: DocumentChecked,
  },
  {
    path: '/documents',
    name: '文档管理',
    icon: Document,
  },
  {
    path: '/settings',
    name: '系统设置',
    icon: Setting,
  },
])

/** 当前激活的菜单 */
const activeMenu = computed(() => route.path)

/** 侧边栏宽度 */
const sidebarWidth = computed(() => (isCollapsed.value ? '64px' : '240px'))
</script>

<template>
  <aside class="sidebar" :class="{ 'is-collapsed': isCollapsed, 'is-mobile-open': isMobileOpen }">
    <!-- 移动端遮罩层 -->
    <div v-if="isMobileOpen" class="sidebar-overlay" @click="closeMobileSidebar"></div>
    <!-- Logo区域 -->
    <div class="sidebar-header">
      <div class="logo">
        <img src="/admin.svg" alt="Logo" class="logo-icon" />
        <span v-show="!isCollapsed" class="logo-text">Seredeli Admin</span>
      </div>
    </div>

    <!-- 菜单区域 -->
    <nav class="sidebar-nav scrollbar-thin">
      <ul class="menu-list">
        <li v-for="item in menuList" :key="item.path" class="menu-item">
          <RouterLink
            :to="item.path"
            class="menu-link"
            :class="{ 'is-active': activeMenu === item.path }"
            :title="isCollapsed ? item.name : ''"
          >
            <component :is="item.icon" class="menu-icon" />
            <span v-show="!isCollapsed" class="menu-text">{{ item.name }}</span>
          </RouterLink>
        </li>
      </ul>
    </nav>

  </aside>
</template>

<style scoped>
.sidebar {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: v-bind(sidebarWidth);
  background-color: var(--sidebar-bg);
  border-right: 1px solid var(--sidebar-border);
  display: flex;
  flex-direction: column;
  transition: width var(--transition-normal);
  z-index: var(--z-fixed);
}

/* Logo区域 */
.sidebar-header {
  flex-shrink: 0;
  padding: var(--spacing-4);
  border-bottom: 1px solid var(--sidebar-border);
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-3);
  height: 40px;
}

.sidebar.is-collapsed .logo {
  justify-content: center;
}

.logo-icon {
  width: 18px;
  height: 18px;
  object-fit: contain;
  filter: var(--logo-filter);
  transition: filter var(--transition-fast);
}

.logo-text {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
}

/* 菜单区域 */
.sidebar-nav {
  flex: 1;
  padding: var(--spacing-3);
  overflow-y: auto;
  overflow-x: hidden;
}

.menu-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
}

.menu-link {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-3) var(--spacing-4);
  color: var(--sidebar-text);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.menu-link:hover {
  background-color: var(--sidebar-bg-hover);
  color: var(--text-primary);
}

.menu-link.is-active {
  background-color: var(--sidebar-bg-active);
  color: var(--sidebar-text-active);
}

.menu-icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
}

.menu-text {
  opacity: 1;
  transition: opacity var(--transition-fast);
}

/* 折叠状态 */
.sidebar.is-collapsed .menu-link {
  justify-content: center;
  padding: var(--spacing-3);
}

/* 移动端浮层模式 */
@media (max-width: 1023px) {
  .sidebar {
    width: 240px;
    transform: translateX(-100%);
    box-shadow: var(--shadow-lg);
  }

  .sidebar.is-mobile-open {
    transform: translateX(0);
  }

  .sidebar-overlay {
    position: fixed;
    inset: 0;
    background-color: rgb(0 0 0 / 0.5);
    z-index: calc(var(--z-fixed) - 1);
  }
}
</style>
