<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useThemeStore } from '@/stores/theme'
import { useUserStore } from '@/stores/user'
import FloatingPanel from '@/components/common/FloatingPanel.vue'
import { getCurrentUser, updateUser, changePassword } from '@/api'
import type { User as UserType } from '@/api'
import {
  Menu,
  Bell,
  FullScreen,
  Moon,
  Sunny,
  ArrowDown,
  User,
  Setting,
  SwitchButton,
} from '@element-plus/icons-vue'

/**
 * 顶部导航栏组件
 * 提供搜索、通知、主题切换、用户菜单等功能
 */

/** 定义props */
const props = defineProps<{
  collapsed?: boolean
}>()

/** 定义事件 */
const emit = defineEmits<{
  'toggle-sidebar': []
}>()

/** 路由 */
const router = useRouter()

/** 用户状态 */
const userStore = useUserStore()

/** 侧边栏宽度 */
const sidebarWidth = computed(() => (props.collapsed ? '64px' : '240px'))

/** 主题状态 */
const themeStore = useThemeStore()

/** 用户下拉菜单显示状态 */
const showUserDropdown = ref(false)

/** 通知数量 */
const notificationCount = ref(3)

/** 通知浮窗显示状态 */
const showNotificationPanel = ref(false)

/** 个人中心浮窗显示状态 */
const showProfilePanel = ref(false)

/** 账号设置浮窗显示状态 */
const showSettingsPanel = ref(false)

/** 当前用户详细信息 */
const currentUser = ref<UserType | null>(null)

/** 加载状态 */
const isLoading = ref(false)

/** 用户名修改消息 */
const usernameMessage = ref({ type: '' as 'success' | 'error' | '', text: '' })

/** 密码修改消息 */
const passwordMessage = ref({ type: '' as 'success' | 'error' | '', text: '' })

/** 个人中心加载错误信息 */
const profileError = ref('')

/** 用户显示名称 */
const displayName = computed(() => currentUser.value?.username || userStore.displayName || 'Admin')

/** 用户角色显示 */
const userRoleDisplay = computed(() => {
  const role = currentUser.value?.role || userStore.user?.role
  switch (role) {
    case 'super_admin':
      return '超级管理员'
    case 'admin':
      return '管理员'
    default:
      return '用户'
  }
})

/** 用户状态显示 */
const userStatusDisplay = computed(() => {
  const status = currentUser.value?.status || userStore.user?.status
  switch (status) {
    case 'online':
      return '在线'
    case 'away':
      return '离开'
    default:
      return '离线'
  }
})

/** 表单数据 */
const usernameForm = ref({
  username: ''
})

const passwordForm = ref({
  old_password: '',
  new_password: '',
  confirm_password: ''
})

/** 密码强度检查结果 */
const passwordStrength = computed(() => {
  const password = passwordForm.value.new_password
  if (!password) {
    return { valid: false, message: '', checks: { length: false, upper: false, lower: false, digit: false } }
  }

  const checks = {
    length: password.length >= 8 && password.length <= 128,
    upper: /[A-Z]/.test(password),
    lower: /[a-z]/.test(password),
    digit: /\d/.test(password)
  }

  const valid = checks.length && checks.upper && checks.lower && checks.digit

  if (valid) {
    return { valid, message: '密码强度符合要求', checks }
  }

  const missing = []
  if (!checks.length) missing.push('长度8-128位')
  if (!checks.upper) missing.push('大写字母')
  if (!checks.lower) missing.push('小写字母')
  if (!checks.digit) missing.push('数字')

  return { valid, message: `还需包含: ${missing.join(', ')}`, checks }
})

/** 用户头像 */
const avatarText = computed(() => {
  const name = userStore.displayName
  return name ? name.charAt(0).toUpperCase() : 'A'
})

/**
 * 切换主题
 */
const toggleTheme = () => {
  themeStore.toggleTheme()
}

/**
 * 切换全屏
 */
const toggleFullscreen = () => {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen()
  } else {
    document.exitFullscreen()
  }
}

/**
 * 切换侧边栏
 */
const toggleSidebar = () => {
  emit('toggle-sidebar')
}

/**
 * 处理退出登录
 * 调用 UserStore 的 logout 方法，清除登录状态并跳转到登录页
 */
const handleLogout = async () => {
  // 关闭下拉菜单
  showUserDropdown.value = false

  // 执行登出
  await userStore.logout()

  // 跳转到登录页
  router.push('/login')
}

/**
 * 获取当前用户信息
 */
const fetchCurrentUser = async () => {
  isLoading.value = true
  profileError.value = ''
  try {
    const response = await getCurrentUser()
    if (response.success && response.data) {
      currentUser.value = response.data
      // 同步更新 store 中的用户信息
      userStore.updateUser(response.data)
    } else {
      profileError.value = response.message || '获取用户信息失败'
    }
  } catch (error) {
    profileError.value = error instanceof Error ? error.message : '网络错误'
  } finally {
    isLoading.value = false
  }
}

/**
 * 打开个人中心浮窗
 */
const openProfilePanel = async () => {
  showProfilePanel.value = true
  showUserDropdown.value = false
  await fetchCurrentUser()
}

/**
 * 打开账号设置浮窗
 */
const openSettingsPanel = async () => {
  showSettingsPanel.value = true
  showUserDropdown.value = false
  // 清空消息
  usernameMessage.value = { type: '', text: '' }
  passwordMessage.value = { type: '', text: '' }
  // 重置表单
  usernameForm.value.username = userStore.user?.username || ''
  passwordForm.value = { old_password: '', new_password: '', confirm_password: '' }
  await fetchCurrentUser()
  if (currentUser.value) {
    usernameForm.value.username = currentUser.value.username
  }
}

/**
 * 更新用户名
 */
const handleUpdateUsername = async () => {
  if (!usernameForm.value.username.trim()) {
    usernameMessage.value = { type: 'error', text: '用户名不能为空' }
    return
  }

  isLoading.value = true
  usernameMessage.value = { type: '', text: '' }

  try {
    const response = await updateUser({ username: usernameForm.value.username.trim() })
    if (response.success && response.data) {
      currentUser.value = response.data
      userStore.updateUser({ username: response.data.username })
      usernameMessage.value = { type: 'success', text: '用户名修改成功，5秒后需要重新登录' }
      // 5秒后登出
      setTimeout(() => {
        userStore.logout()
        router.push('/login')
      }, 5000)
    } else {
      usernameMessage.value = { type: 'error', text: response.message || '修改失败' }
    }
  } catch (error) {
    usernameMessage.value = { type: 'error', text: error instanceof Error ? error.message : '网络错误' }
  } finally {
    isLoading.value = false
  }
}

/**
 * 验证密码强度
 * - 最小长度：8字符
 * - 必须包含：大写字母、小写字母、数字
 * @param password 密码
 * @returns 验证结果，成功返回 null，失败返回错误信息
 */
const validatePasswordStrength = (password: string): string | null => {
  if (password.length < 8) {
    return '密码长度至少为8个字符'
  }
  if (password.length > 128) {
    return '密码长度不能超过128个字符'
  }

  // 检查是否包含大写字母
  const hasUpper = /[A-Z]/.test(password)
  // 检查是否包含小写字母
  const hasLower = /[a-z]/.test(password)
  // 检查是否包含数字
  const hasDigit = /\d/.test(password)

  if (!hasUpper || !hasLower || !hasDigit) {
    return '密码必须包含至少一个大写字母、一个小写字母和一个数字'
  }

  return null
}

/**
 * 修改密码
 */
const handleChangePassword = async () => {
  // 验证表单
  if (!passwordForm.value.old_password) {
    passwordMessage.value = { type: 'error', text: '请输入当前密码' }
    return
  }
  if (!passwordForm.value.new_password) {
    passwordMessage.value = { type: 'error', text: '请输入新密码' }
    return
  }

  // 验证密码强度
  const passwordError = validatePasswordStrength(passwordForm.value.new_password)
  if (passwordError) {
    passwordMessage.value = { type: 'error', text: passwordError }
    return
  }

  if (passwordForm.value.new_password !== passwordForm.value.confirm_password) {
    passwordMessage.value = { type: 'error', text: '两次输入的新密码不一致' }
    return
  }

  isLoading.value = true
  passwordMessage.value = { type: '', text: '' }

  try {
    const response = await changePassword({
      old_password: passwordForm.value.old_password,
      new_password: passwordForm.value.new_password
    })
    if (response.success) {
      passwordMessage.value = { type: 'success', text: '密码修改成功，5秒后需要重新登录' }
      // 清空密码表单
      passwordForm.value = { old_password: '', new_password: '', confirm_password: '' }
      // 5秒后登出
      setTimeout(() => {
        userStore.logout()
        router.push('/login')
      }, 5000)
    } else {
      passwordMessage.value = { type: 'error', text: response.message || '修改失败' }
    }
  } catch (error) {
    passwordMessage.value = { type: 'error', text: error instanceof Error ? error.message : '网络错误' }
  } finally {
    isLoading.value = false
  }
}

/**
 * 格式化日期时间
 */
const formatDateTime = (dateStr: string | undefined): string => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 组件挂载时获取用户信息
onMounted(() => {
  fetchCurrentUser()
})
</script>

<template>
  <header class="header">
    <!-- 左侧：面包屑/标题 -->
    <div class="header-left">
      <h1 class="page-title">管理后台</h1>
    </div>

    <!-- 右侧：工具栏 -->
    <div class="header-right">
      <!-- 主题切换 -->
      <button class="toolbar-btn" @click="toggleTheme">
        <Moon v-if="themeStore.isDark" />
        <Sunny v-else />
      </button>

      <!-- 全屏 -->
      <button class="toolbar-btn" @click="toggleFullscreen">
        <FullScreen />
      </button>

      <!-- 通知 -->
      <button class="toolbar-btn notification-btn" @click="showNotificationPanel = true">
        <Bell />
        <span v-if="notificationCount > 0" class="notification-badge">{{ notificationCount }}</span>
      </button>

      <!-- 侧边栏折叠按钮 -->
      <button class="toolbar-btn sidebar-toggle-btn" @click="toggleSidebar">
        <Menu />
      </button>

      <!-- 用户菜单 -->
      <div class="user-menu" :class="{ 'is-open': showUserDropdown }">
        <button class="user-trigger" @click="showUserDropdown = !showUserDropdown">
          <div class="avatar">{{ avatarText }}</div>
          <span class="username">{{ displayName }}</span>
          <ArrowDown class="arrow-icon" />
        </button>

        <div class="dropdown-menu">
          <div class="dropdown-item" @click="openProfilePanel">
            <User />
            <span>个人中心</span>
          </div>
          <div class="dropdown-item" @click="openSettingsPanel">
            <Setting />
            <span>账号设置</span>
          </div>
          <div class="dropdown-divider"></div>
          <div class="dropdown-item" @click="handleLogout">
            <SwitchButton />
            <span>退出登录</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 通知浮窗 -->
    <FloatingPanel
      v-model:visible="showNotificationPanel"
      title="待办通知"
      width="480px"
    >
      <div class="notification-list">
        <div class="notification-item">
          <div class="notification-icon warning">
            <Bell />
          </div>
          <div class="notification-content">
            <p class="notification-title">配置变更需要确认</p>
            <p class="notification-desc">系统配置已修改，需要重启服务生效</p>
            <span class="notification-time">10分钟前</span>
          </div>
          <div class="notification-actions">
            <button class="btn-approve">确认</button>
            <button class="btn-snooze">稍后</button>
          </div>
        </div>
        <div class="notification-item">
          <div class="notification-icon info">
            <Bell />
          </div>
          <div class="notification-content">
            <p class="notification-title">安全告警待处理</p>
            <p class="notification-desc">检测到异常登录行为，需要管理员确认</p>
            <span class="notification-time">1小时前</span>
          </div>
          <div class="notification-actions">
            <button class="btn-approve">确认</button>
            <button class="btn-snooze">稍后</button>
          </div>
        </div>
      </div>
    </FloatingPanel>

    <!-- 个人中心浮窗 -->
    <FloatingPanel
      v-model:visible="showProfilePanel"
      title="个人中心"
      width="400px"
    >
      <div class="profile-content">
        <!-- 加载状态 -->
        <div v-if="isLoading" class="loading-state">
          <span>加载中...</span>
        </div>
        <!-- 错误提示 -->
        <div v-else-if="profileError" class="error-state">
          <span>{{ profileError }}</span>
        </div>
        <!-- 用户信息 -->
        <template v-else>
          <div class="profile-header">
            <div class="profile-avatar">{{ avatarText }}</div>
            <div class="profile-info">
              <h4 class="profile-name">{{ displayName }}</h4>
              <p class="profile-role">{{ userRoleDisplay }}</p>
            </div>
          </div>
          <div class="profile-details">
            <div class="profile-item">
              <span class="profile-label">用户ID</span>
              <span class="profile-value">{{ currentUser?.id || '-' }}</span>
            </div>
            <div class="profile-item">
              <span class="profile-label">用户名</span>
              <span class="profile-value">{{ currentUser?.username || userStore.user?.username || '-' }}</span>
            </div>
            <div class="profile-item">
              <span class="profile-label">邮箱</span>
              <span class="profile-value">{{ currentUser?.email || userStore.user?.email || '-' }}</span>
            </div>
            <div class="profile-item">
              <span class="profile-label">角色</span>
              <span class="profile-value">{{ userRoleDisplay }}</span>
            </div>
            <div class="profile-item">
              <span class="profile-label">状态</span>
              <span class="profile-value">{{ userStatusDisplay }}</span>
            </div>
            <div class="profile-item">
              <span class="profile-label">注册时间</span>
              <span class="profile-value">{{ formatDateTime(currentUser?.created_at) }}</span>
            </div>
          </div>
        </template>
      </div>
    </FloatingPanel>

    <!-- 账号设置浮窗 -->
    <FloatingPanel
      v-model:visible="showSettingsPanel"
      title="账号设置"
      width="420px"
    >
      <div class="settings-content">
        <div class="settings-section">
          <h4 class="settings-section-title">修改用户名</h4>
          <!-- 用户名修改消息提示 -->
          <div v-if="usernameMessage.type === 'success'" class="alert alert-success">
            {{ usernameMessage.text }}
          </div>
          <div v-if="usernameMessage.type === 'error'" class="alert alert-error">
            {{ usernameMessage.text }}
          </div>
          <div class="form-group">
            <label class="form-label">新用户名</label>
            <input
              v-model="usernameForm.username"
              type="text"
              class="form-input"
              placeholder="输入新用户名"
              :disabled="isLoading"
            />
          </div>
          <button
            class="btn-submit"
            :disabled="isLoading"
            @click="handleUpdateUsername"
          >
            {{ isLoading ? '保存中...' : '保存修改' }}
          </button>
        </div>
        <div class="settings-divider"></div>
        <div class="settings-section">
          <h4 class="settings-section-title">修改密码</h4>
          <!-- 密码修改消息提示 -->
          <div v-if="passwordMessage.type === 'success'" class="alert alert-success">
            {{ passwordMessage.text }}
          </div>
          <div v-if="passwordMessage.type === 'error'" class="alert alert-error">
            {{ passwordMessage.text }}
          </div>
          <div class="form-group">
            <label class="form-label">当前密码</label>
            <input
              v-model="passwordForm.old_password"
              type="password"
              class="form-input"
              placeholder="输入当前密码"
              :disabled="isLoading"
            />
          </div>
          <div class="form-group">
            <label class="form-label">新密码</label>
            <input
              v-model="passwordForm.new_password"
              type="password"
              class="form-input"
              :class="{ 'is-valid': passwordStrength.valid, 'is-invalid': passwordForm.new_password && !passwordStrength.valid }"
              placeholder="输入新密码（至少8位）"
              :disabled="isLoading"
            />
            <!-- 密码强度提示 -->
            <div v-if="passwordForm.new_password" class="password-strength">
              <div class="strength-indicators">
                <span
                  v-for="(valid, key) in passwordStrength.checks"
                  :key="key"
                  class="strength-item"
                  :class="{ valid }"
                >
                  {{ key === 'length' ? '✓ 长度8-128位' : key === 'upper' ? '✓ 大写字母' : key === 'lower' ? '✓ 小写字母' : '✓ 数字' }}
                </span>
              </div>
              <p class="strength-message" :class="{ valid: passwordStrength.valid }">
                {{ passwordStrength.message }}
              </p>
            </div>
          </div>
          <div class="form-group">
            <label class="form-label">确认新密码</label>
            <input
              v-model="passwordForm.confirm_password"
              type="password"
              class="form-input"
              placeholder="再次输入新密码"
              :disabled="isLoading"
            />
          </div>
          <button
            class="btn-submit"
            :disabled="isLoading"
            @click="handleChangePassword"
          >
            {{ isLoading ? '修改中...' : '修改密码' }}
          </button>
        </div>
      </div>
    </FloatingPanel>
  </header>
</template>

<style scoped>
.header {
  position: fixed;
  top: 0;
  right: 0;
  left: v-bind(sidebarWidth);
  height: var(--header-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-6);
  background-color: var(--header-bg);
  border-bottom: 1px solid var(--header-border);
  transition: left var(--transition-normal);
  z-index: calc(var(--z-fixed) - 1);
}

/* 左侧 */
.header-left {
  display: flex;
  align-items: center;
}

.page-title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--header-text);
}

/* 右侧工具栏 */
.header-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
}

.toolbar-btn {
  position: relative;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  background-color: transparent;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.toolbar-btn svg {
  width: 18px;
  height: 18px;
}

.toolbar-btn:hover {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

.notification-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: var(--font-weight-bold);
  color: white;
  background-color: var(--error);
  border-radius: var(--radius-full);
}

/* 用户菜单 */
.user-menu {
  position: relative;
  margin-left: var(--spacing-2);
}

.user-trigger {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-1) var(--spacing-2) var(--spacing-1) var(--spacing-1);
  background-color: transparent;
  border-radius: var(--radius-md);
  transition: background-color var(--transition-fast);
}

.user-trigger:hover {
  background-color: var(--bg-secondary);
}

.avatar {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--primary-alpha);
  color: var(--primary);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  border-radius: var(--radius-full);
}

.username {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-primary);
}

.arrow-icon {
  width: 16px;
  height: 16px;
  color: var(--text-tertiary);
  transition: transform var(--transition-fast);
}

.user-menu.is-open .arrow-icon {
  transform: rotate(180deg);
}

/* 下拉菜单 */
.dropdown-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 180px;
  padding: var(--spacing-2);
  background-color: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  opacity: 0;
  visibility: hidden;
  transform: translateY(-8px);
  transition: all var(--transition-fast);
}

.user-menu.is-open .dropdown-menu {
  opacity: 1;
  visibility: visible;
  transform: translateY(0);
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-2) var(--spacing-3);
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.dropdown-item svg {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.dropdown-item:hover {
  background-color: var(--bg-secondary);
}

.dropdown-divider {
  height: 1px;
  margin: var(--spacing-2) 0;
  background-color: var(--border-secondary);
}

/* ========== 浮窗内容样式 ========== */

/* 通知列表 */
.notification-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.notification-item {
  display: flex;
  gap: var(--spacing-3);
  padding: var(--spacing-4);
  background-color: var(--bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-secondary);
}

.notification-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.notification-icon.warning {
  background-color: var(--warning-alpha, rgba(245, 158, 11, 0.1));
  color: var(--warning, #f59e0b);
}

.notification-icon.info {
  background-color: var(--info-alpha, rgba(59, 130, 246, 0.1));
  color: var(--info, #3b82f6);
}

.notification-icon svg {
  width: 20px;
  height: 20px;
}

.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-title {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-1);
}

.notification-desc {
  font-size: var(--font-size-xs);
  color: var(--text-secondary);
  margin-bottom: var(--spacing-2);
  line-height: var(--line-height-normal);
}

.notification-time {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
}

.notification-actions {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
  flex-shrink: 0;
}

.btn-approve,
.btn-snooze {
  padding: var(--spacing-1) var(--spacing-3);
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  border-radius: var(--radius-md);
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-approve {
  background-color: var(--primary);
  color: white;
}

.btn-approve:hover {
  background-color: var(--primary-hover);
}

.btn-snooze {
  background-color: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-primary);
}

.btn-snooze:hover {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

/* 个人中心 */
.profile-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-6);
}

.profile-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  padding-bottom: var(--spacing-4);
  border-bottom: 1px solid var(--border-secondary);
}

.profile-avatar {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--primary-alpha);
  color: var(--primary);
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  border-radius: var(--radius-full);
}

.profile-info {
  flex: 1;
}

.profile-name {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-1);
}

.profile-role {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.profile-details {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.profile-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-3) 0;
  border-bottom: 1px solid var(--border-secondary);
}

.profile-item:last-child {
  border-bottom: none;
}

.profile-label {
  font-size: var(--font-size-sm);
  color: var(--text-secondary);
}

.profile-value {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-primary);
}

/* 加载和错误状态 */
.loading-state,
.error-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-8);
  font-size: var(--font-size-sm);
}

.loading-state {
  color: var(--text-secondary);
}

.error-state {
  color: var(--error);
}

/* 提示消息 */
.alert {
  padding: var(--spacing-3) var(--spacing-4);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  margin-bottom: var(--spacing-4);
}

.alert-success {
  background-color: var(--success-alpha, rgba(34, 197, 94, 0.1));
  color: var(--success, #22c55e);
  border: 1px solid var(--success, #22c55e);
}

.alert-error {
  background-color: var(--error-alpha, rgba(239, 68, 68, 0.1));
  color: var(--error, #ef4444);
  border: 1px solid var(--error, #ef4444);
}

/* 密码强度提示 */
.password-strength {
  margin-top: var(--spacing-2);
}

.strength-indicators {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-2);
  margin-bottom: var(--spacing-2);
}

.strength-item {
  font-size: var(--font-size-xs);
  color: var(--text-muted);
  padding: var(--spacing-1) var(--spacing-2);
  background-color: var(--bg-secondary);
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.strength-item.valid {
  color: var(--success, #22c55e);
  background-color: var(--success-alpha, rgba(34, 197, 94, 0.1));
}

.strength-message {
  font-size: var(--font-size-xs);
  color: var(--error, #ef4444);
  margin: 0;
}

.strength-message.valid {
  color: var(--success, #22c55e);
}

/* 表单输入框状态 */
.form-input.is-valid {
  border-color: var(--success, #22c55e);
}

.form-input.is-invalid {
  border-color: var(--error, #ef4444);
}

/* 账号设置 */
.settings-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-6);
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.settings-section-title {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--spacing-2);
}

.settings-divider {
  height: 1px;
  background-color: var(--border-secondary);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-2);
}

.form-label {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-secondary);
}

.form-input {
  padding: var(--spacing-2) var(--spacing-3);
  font-size: var(--font-size-sm);
  color: var(--text-primary);
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  border-color: var(--primary);
}

.form-input::placeholder {
  color: var(--text-muted);
}

.btn-submit {
  padding: var(--spacing-2) var(--spacing-4);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: white;
  background-color: var(--primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color var(--transition-fast);
  align-self: flex-start;
}

.btn-submit:hover {
  background-color: var(--primary-hover);
}

/* 响应式 */
@media (max-width: 1023px) {
  .header {
    left: 0 !important;
  }

  .username {
    display: none;
  }
}

@media (max-width: 767px) {
  .header {
    padding: 0 var(--spacing-4);
  }

  .header-center {
    display: none;
  }

  .toolbar-btn {
    width: 36px;
    height: 36px;
  }

  .header-right {
    gap: var(--spacing-1);
  }

  .user-menu {
    margin-left: var(--spacing-1);
  }

  .user-trigger {
    padding: var(--spacing-1);
  }
}
</style>
