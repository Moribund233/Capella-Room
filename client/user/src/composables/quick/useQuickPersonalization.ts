import { computed } from 'vue'
import { usePersonalizationStore } from '@/stores/personalization'
import { useThemeStore } from '@/stores/theme'
import { useGlobalModal } from '@/composables/useGlobalModal'
import PersonalizationModal from '@/components/quick/PersonalizationModal.vue'
import type { QuickItem, QuickRuntimeItem } from '@/config/quick'

/**
 * 个性化 Quick 组合式函数
 * @param config Quick 配置项
 * @returns Quick 运行时属性
 */
export function useQuickPersonalization(config: QuickItem): Partial<QuickRuntimeItem> {
  const personalizationStore = usePersonalizationStore()
  const themeStore = useThemeStore()
  const { open, close } = useGlobalModal()

  /**
   * 当前图标根据是否有自定义设置切换
   */
  const currentIcon = computed(() => {
    // 如果有自定义设置，显示不同的图标或保持原样
    if (personalizationStore.hasCustomAccent || personalizationStore.hasCustomBackground) {
      return config.iconAlt || config.icon
    }
    return config.icon
  })

  /**
   * 打开个性化设置弹窗
   */
  function openPersonalizationModal() {
    open({
      title: '个性化设置',
      component: PersonalizationModal,
      componentProps: {
        modelValue: personalizationStore.config,
        'onUpdate:modelValue': (value: typeof personalizationStore.config) => {
          // 更新主题
          if (value.theme !== themeStore.themeSetting) {
            themeStore.setTheme(value.theme)
          }
          // 更新其他配置
          personalizationStore.updateConfig(value)
        },
        onConfirm: () => {
          close()
        },
        onReset: () => {
          // 重置主题
          themeStore.resetToSystem()
          // 重置个性化配置
          personalizationStore.resetToDefault()
        },
      },
      preset: 'card',
      maskClosable: true,
      closable: true,
    })
  }

  return {
    isActive: computed(() => !!(personalizationStore.hasCustomAccent || personalizationStore.hasCustomBackground)),
    currentIcon: computed(() => currentIcon.value),
    disabled: false,
    onClick: openPersonalizationModal,
  }
}
