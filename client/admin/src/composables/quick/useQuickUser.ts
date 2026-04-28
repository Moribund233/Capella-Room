import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/store'
import { useGlobalModal } from '@/composables/useGlobalModal'
import AboutModal from '@/components/quick/AboutModal.vue'
import LoginModalContent from '@/components/quick/LoginModalContent.vue'
import UserProfileModal from '@/components/quick/UserProfileModal.vue'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

/**
 * 用户中心 Quick 组合式函数
 *
 * 功能：
 * - 显示用户登录状态
 * - 提供用户相关操作入口（用户详情、关于、帮助、登出）
 * - 支持 action 类型直接打开登录/用户面板
 * - 支持 menu 类型显示下拉菜单
 * - 关于弹窗内部处理，直接显示应用信息
 *
 * @param config Quick 配置项
 * @param context Quick 上下文
 * @returns Quick 运行时接口
 *
 * @example
 * // ui.ts 配置 - 下拉菜单模式（推荐）
 * {
 *   key: 'user',
 *   display: 'dropdown',
 *   type: 'menu',
 *   icon: 'UserCircle',
 *   iconAlt: 'User',
 *   label: '用户中心',
 *   children: [
 *     { key: 'profile', label: '用户详情', icon: 'User' },
 *     { key: 'about', label: '关于', icon: 'Info' },
 *     { key: 'help', label: '帮助', icon: 'HelpCircle' },
 *     { key: 'logout', label: '登出', icon: 'LogOut' }
 *   ]
 * }
 *
 * @example
 * // ui.ts 配置 - 直接点击模式
 * {
 *   key: 'user',
 *   display: 'visible',
 *   type: 'action',
 *   icon: 'UserCircle',
 *   iconAlt: 'User',
 *   label: '用户中心',
 * }
 */
export function useQuickUser(config: QuickConfigItem, context: QuickContext): UseQuickReturn {
  const authStore = useAuthStore()
  const router = useRouter()
  const { open, close, warning } = useGlobalModal()

  /**
   * 是否已登录
   * 作为功能状态，控制图标显示和可用操作
   */
  const isActive = computed(() => authStore.isLoggedIn)

  /**
   * 当前图标
   * 已登录时显示 iconAlt（用户图标），未登录时显示 icon（用户圆圈图标）
   */
  const currentIcon = computed(() => (isActive.value ? config.iconAlt || config.icon : config.icon))

  /**
   * 显示关于弹窗
   */
  function showAboutModal(): void {
    open({
      title: '',
      component: AboutModal,
      componentProps: {},
      preset: 'card',
      width: 400,
      closable: true,
      maskClosable: true,
    })
  }

  /**
   * 显示登录弹窗
   */
  function showLoginModal(): void {
    open({
      title: '',
      component: LoginModalContent,
      componentProps: {
        onSuccess: () => {
          // 登录成功，关闭弹窗
          close()
        },
      },
      preset: 'card',
      width: 400,
      closable: true,
      maskClosable: false,
    })
  }

  /**
   * 显示用户详情弹窗
   */
  function showUserProfileModal(): void {
    open({
      title: '用户详情',
      component: UserProfileModal,
      componentProps: {
        userInfo: authStore.userInfo,
      },
      preset: 'card',
      width: 400,
      closable: true,
      maskClosable: true,
    })
  }

  /**
   * 点击主按钮处理
   * - action 类型：已登录时打开用户面板，未登录时触发登录
   * - menu 类型：不处理，由下拉菜单触发 onSelect
   */
  function onClick(): void {
    if (config.type === 'menu') {
      // menu 类型：不处理，由下拉菜单触发 onSelect
      return
    }

    // action 类型：根据登录状态执行不同操作
    if (isActive.value) {
      // 已登录：打开用户面板
      context.emitAction(config.key, 'open-panel')
    } else {
      // 未登录：显示登录弹窗
      showLoginModal()
    }
  }

  /**
   * 选择子菜单项处理
   * 根据子菜单 key 和登录状态执行相应操作
   */
  function onSelect(childKey: string): void {
    // 关于弹窗：无需登录，内部直接处理
    if (childKey === 'about') {
      showAboutModal()
      return
    }

    // 帮助：无需登录，触发外部处理
    if (childKey === 'help') {
      context.emitAction(config.key, childKey)
      return
    }

    // 以下功能需要登录状态检查
    const needLoginKeys = ['profile', 'logout']

    if (needLoginKeys.includes(childKey)) {
      // 未登录时显示登录弹窗
      if (!isActive.value) {
        showLoginModal()
        return
      }

      // 已登录，处理具体逻辑
      if (childKey === 'profile') {
        // 用户详情：显示用户详情弹窗
        showUserProfileModal()
        return
      }

      if (childKey === 'logout') {
        // 登出操作：显示确认弹窗
        warning({
          title: '确认退出',
          content: '确定要退出登录吗？',
          positiveText: '确认退出',
          negativeText: '取消',
          onPositiveClick: async () => {
            // 执行登出操作
            authStore.logout()
            // 触发外部事件
            context.emitAction(config.key, childKey)
            // 主动跳转到登录页
            await router.push({ name: 'Login' })
          },
        })
        return
      }

      // profile 等已登录操作
      context.emitAction(config.key, childKey)
    }
  }

  return {
    isActive,
    currentIcon,
    onClick,
    onSelect: config.type === 'menu' ? onSelect : undefined,
  }
}
