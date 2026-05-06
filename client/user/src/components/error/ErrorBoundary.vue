<script setup lang="ts">
import { ref, onErrorCaptured, type ComponentPublicInstance } from 'vue'
import { AlertTriangle, RefreshCw, Home } from 'lucide-vue-next'
import { useRouter } from 'vue-router'

/**
 * 错误边界组件
 * 捕获子组件树中的错误，防止整个应用崩溃
 */

const props = withDefaults(
  defineProps<{
    /** 是否显示默认错误UI */
    showDefaultUI?: boolean
    /** 是否自动重置错误 */
    autoReset?: boolean
    /** 自动重置时间（毫秒） */
    autoResetDelay?: number
  }>(),
  {
    showDefaultUI: true,
    autoReset: false,
    autoResetDelay: 5000,
  }
)

const emit = defineEmits<{
  /** 错误捕获事件 */
  (e: 'error', error: Error, instance: ComponentPublicInstance | null, info: string): void
  /** 错误重置事件 */
  (e: 'reset'): void
}>()

const router = useRouter()
const hasError = ref(false)
const errorMessage = ref('')
const errorInfo = ref('')

/**
 * 捕获错误
 */
onErrorCaptured((err, instance, info) => {
  const error = err instanceof Error ? err : new Error(String(err))

  hasError.value = true
  errorMessage.value = error.message || '组件发生错误'
  errorInfo.value = info

  // 记录错误日志
  console.error('[ErrorBoundary] 捕获到错误:', error)
  console.error('[ErrorBoundary] 组件实例:', instance)
  console.error('[ErrorBoundary] 错误信息:', info)

  emit('error', error, instance, info)

  // 自动重置
  if (props.autoReset) {
    setTimeout(() => {
      resetError()
    }, props.autoResetDelay)
  }

  // 阻止错误继续传播
  return false
})

/**
 * 重置错误状态
 */
function resetError() {
  hasError.value = false
  errorMessage.value = ''
  errorInfo.value = ''
  emit('reset')
}

/**
 * 返回首页
 */
function goHome() {
  router.push('/')
  resetError()
}
</script>

<template>
  <div class="error-boundary">
    <!-- 错误状态显示 -->
    <div
      v-if="hasError && showDefaultUI"
      class="error-boundary__fallback"
      role="alert"
      aria-live="assertive"
    >
      <div class="error-boundary__content">
        <div class="error-boundary__icon">
          <AlertTriangle :size="48" />
        </div>

        <h2 class="error-boundary__title">出错了</h2>

        <p class="error-boundary__message">
          {{ errorMessage }}
        </p>

        <p v-if="errorInfo" class="error-boundary__info">
          {{ errorInfo }}
        </p>

        <div class="error-boundary__actions">
          <button
            class="error-boundary__btn error-boundary__btn--primary"
            @click="resetError"
          >
            <RefreshCw :size="16" />
            <span>重试</span>
          </button>

          <button
            class="error-boundary__btn error-boundary__btn--secondary"
            @click="goHome"
          >
            <Home :size="16" />
            <span>返回首页</span>
          </button>
        </div>
      </div>
    </div>

    <!-- 正常内容显示 -->
    <slot v-else />
  </div>
</template>

<style scoped>
.error-boundary {
  width: 100%;
  height: 100%;
}

.error-boundary__fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 24px;
  background: var(--error-bg, #fff2f0);
  border: 1px solid var(--error-border, #ffccc7);
  border-radius: 8px;
}

.error-boundary__content {
  text-align: center;
  max-width: 400px;
}

.error-boundary__icon {
  color: var(--error-color, #ff4d4f);
  margin-bottom: 16px;
}

.error-boundary__title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary, rgba(0, 0, 0, 0.88));
  margin: 0 0 12px;
}

.error-boundary__message {
  font-size: 14px;
  color: var(--text-secondary, rgba(0, 0, 0, 0.65));
  margin: 0 0 8px;
  line-height: 1.5;
}

.error-boundary__info {
  font-size: 12px;
  color: var(--text-tertiary, rgba(0, 0, 0, 0.45));
  margin: 0 0 24px;
  font-family: monospace;
  background: rgba(0, 0, 0, 0.04);
  padding: 8px;
  border-radius: 4px;
}

.error-boundary__actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

.error-boundary__btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.error-boundary__btn--primary {
  background: var(--primary-color, #1890ff);
  color: white;
}

.error-boundary__btn--primary:hover {
  background: var(--primary-hover, #40a9ff);
}

.error-boundary__btn--secondary {
  background: var(--secondary-bg, rgba(0, 0, 0, 0.04));
  color: var(--text-primary, rgba(0, 0, 0, 0.88));
}

.error-boundary__btn--secondary:hover {
  background: var(--secondary-hover, rgba(0, 0, 0, 0.08));
}
</style>
