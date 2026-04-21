import { computed } from 'vue'
import { useLayoutStore } from '@/store'
import { useGlobalModal } from '@/composables/useGlobalModal'
import PersonalizationModal from '@/components/quick/PersonalizationModal.vue'
import type { QuickConfigItem, QuickContext, UseQuickReturn } from './types'

/**
 * 个性化设置 Quick 组合式函数
 *
 * 功能：
 * - 打开个性化设置弹窗
 * - 支持调整侧边栏、顶部、底部的尺寸、圆角、透明度
 * - 支持重置为默认设置
 *
 * @param config Quick 配置项
 * @param context Quick 上下文
 * @returns Quick 运行时接口
 *
 * @example
 * // ui.ts 配置
 * {
 *   key: 'personalization',
 *   display: 'dropdown',
 *   type: 'menu',
 *   icon: 'Palette',
 *   label: '个性化',
 *   children: [
 *     { key: 'sidebar', label: '侧边栏', icon: 'PanelLeft' },
 *     { key: 'header', label: '顶部', icon: 'PanelTop' },
 *     { key: 'footer', label: '底部', icon: 'PanelBottom' },
 *   ]
 * }
 */
export function useQuickPersonalization(
  config: QuickConfigItem,
  context: QuickContext,
): UseQuickReturn {
  const layoutStore = useLayoutStore()
  const { open, close } = useGlobalModal()

  /**
   * 当前图标
   */
  const currentIcon = computed(() => config.icon)

  /**
   * 是否激活
   */
  const isActive = computed(() => false)

  /**
   * 显示个性化设置弹窗
   * @param activeTab 默认激活的标签页
   */
  function showPersonalizationModal(activeTab: string = 'sidebar'): void {
    open({
      title: '个性化设置',
      component: PersonalizationModal,
      componentProps: {
        modelValue: layoutStore.layoutStyles,
        defaultTab: activeTab,
        'onUpdate:modelValue': (styles: typeof layoutStore.layoutStyles) => {
          layoutStore.updateLayoutStyles(styles)
        },
        onConfirm: () => {
          close()
        },
        onReset: () => {
          layoutStore.resetLayoutStyles()
        },
      },
      preset: 'card',
      width: 560,
      closable: true,
      maskClosable: true,
    })
  }

  /**
   * 点击主按钮处理
   * - action 类型：直接打开个性化设置弹窗
   * - menu 类型：不处理，由下拉菜单触发 onSelect
   */
  function onClick(): void {
    if (config.type === 'action') {
      showPersonalizationModal()
    }
  }

  /**
   * 选择子菜单项处理
   * 根据子菜单 key 打开个性化设置弹窗并切换到对应标签页
   */
  function onSelect(childKey: string): void {
    const validTabs = ['sidebar', 'header', 'footer']
    const activeTab = validTabs.includes(childKey) ? childKey : 'sidebar'
    showPersonalizationModal(activeTab)
  }

  return {
    isActive,
    currentIcon,
    onClick,
    onSelect: config.type === 'menu' ? onSelect : undefined,
  }
}
