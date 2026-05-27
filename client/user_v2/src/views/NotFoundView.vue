<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, HomeFilled } from '@element-plus/icons-vue'

const router = useRouter()
const { t } = useI18n()

/**
 * 返回首页
 */
function goHome() {
  router.push('/')
}

/**
 * 返回上一页
 */
function goBack() {
  router.back()
}
</script>

<template>
  <div class="not-found-page">
    <div class="not-found-content">
      <!-- 404 数字 -->
      <div class="error-code">
        <span class="digit">4</span>
        <img src="/favicon.svg" alt="" class="logo-mark" />
        <span class="digit">4</span>
      </div>

      <!-- 错误信息 -->
      <h1 class="error-title">{{ t('notFound.title') }}</h1>
      <p class="error-description">{{ t('notFound.description') }}</p>

      <!-- 操作按钮 -->
      <div class="error-actions">
        <el-button type="primary" size="large" @click="goHome">
          <el-icon><HomeFilled /></el-icon>
          {{ t('notFound.goHome') }}
        </el-button>
        <el-button size="large" text @click="goBack">
          <el-icon><ArrowLeft /></el-icon>
          {{ t('notFound.goBack') }}
        </el-button>
      </div>

      <!-- 装饰元素 -->
      <div class="decoration">
        <div class="orbit orbit-1"></div>
        <div class="orbit orbit-2"></div>
        <div class="orbit orbit-3"></div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.not-found-page {
  min-height: 100vh;
  display: grid;
  place-items: center;
  background: var(--wave-bg);
  color: var(--wave-fg);
  position: relative;
  overflow: hidden;
}

.not-found-content {
  text-align: center;
  position: relative;
  z-index: 1;
  padding: 40px;
}

.error-code {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-bottom: 32px;
}

.digit {
  font-family: var(--wave-font-display);
  font-size: 120px;
  font-weight: 700;
  line-height: 1;
  background: linear-gradient(135deg, var(--wave-accent), var(--wave-accent-pink));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: float 3s ease-in-out infinite;

  &:first-child {
    animation-delay: 0s;
  }

  &:last-child {
    animation-delay: 0.5s;
  }
}

.logo-mark {
  width: 80px;
  height: 80px;
  border-radius: 20px;
  filter: var(--wave-logo-filter);
  animation: float 3s ease-in-out infinite;
  animation-delay: 0.25s;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.error-title {
  font-family: var(--wave-font-display);
  font-size: 32px;
  font-weight: 600;
  margin: 0 0 12px;
}

.error-description {
  font-size: 16px;
  color: var(--wave-muted);
  margin: 0 0 32px;
  max-width: 400px;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.decoration {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: -1;
}

.orbit {
  position: absolute;
  border: 1px solid var(--wave-border);
  border-radius: 50%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);

  &::before {
    content: '';
    position: absolute;
    width: 8px;
    height: 8px;
    background: var(--wave-accent);
    border-radius: 50%;
    top: -4px;
    left: 50%;
    transform: translateX(-50%);
  }
}

.orbit-1 {
  width: 300px;
  height: 300px;
  animation: rotate 20s linear infinite;

  &::before {
    background: var(--wave-accent);
  }
}

.orbit-2 {
  width: 450px;
  height: 450px;
  animation: rotate 30s linear infinite reverse;

  &::before {
    background: var(--wave-accent-pink);
    top: auto;
    bottom: -4px;
  }
}

.orbit-3 {
  width: 600px;
  height: 600px;
  animation: rotate 40s linear infinite;

  &::before {
    background: var(--wave-accent-green);
    left: -4px;
    top: 50%;
    transform: translateY(-50%);
  }
}

@keyframes rotate {
  from {
    transform: translate(-50%, -50%) rotate(0deg);
  }
  to {
    transform: translate(-50%, -50%) rotate(360deg);
  }
}

@media (max-width: 640px) {
  .digit {
    font-size: 80px;
  }

  .logo-mark {
    width: 60px;
    height: 60px;
    font-size: 30px;
  }

  .error-title {
    font-size: 24px;
  }

  .orbit-1 {
    width: 200px;
    height: 200px;
  }

  .orbit-2 {
    width: 300px;
    height: 300px;
  }

  .orbit-3 {
    width: 400px;
    height: 400px;
  }
}
</style>
