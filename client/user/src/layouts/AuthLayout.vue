<script setup lang="ts">
import { useResponsive } from '@/composables/useResponsive'

const { isMobile } = useResponsive()
</script>

<template>
  <div class="auth-layout">
    <div class="auth-layout__container">
      <div v-if="!isMobile" class="auth-layout__banner">
        <div class="auth-layout__banner-content">
          <img src="/favicon.svg" alt="Seredeli Room" class="auth-layout__logo" />
          <h1 class="auth-layout__title">Seredeli Room</h1>
          <p class="auth-layout__subtitle">现代化实时聊天室</p>
        </div>
      </div>
      <div class="auth-layout__form">
        <router-view />
      </div>
    </div>
  </div>
</template>

<style scoped>
.auth-layout {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, var(--color-primary-light) 0%, var(--color-background) 100%);
}

.auth-layout__container {
  display: flex;
  width: 800px;
  max-width: 90vw;
  min-height: 500px;
  background: var(--color-white);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
  overflow: hidden;
}

.auth-layout__banner {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 360px;
  flex-shrink: 0;
  background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-pressed) 100%);
  color: var(--color-white);
}

.auth-layout__banner-content {
  text-align: center;
  padding: var(--space-3xl);
}

.auth-layout__logo {
  width: 80px;
  height: 80px;
  margin-bottom: var(--space-lg);
  filter: brightness(0) invert(1);
}

.auth-layout__title {
  font-size: var(--font-size-h1);
  font-weight: 600;
  margin-bottom: var(--space-sm);
}

.auth-layout__subtitle {
  font-size: var(--font-size-body);
  opacity: 0.9;
}

.auth-layout__form {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl);
}
</style>
