<template>
  <div class="about-modal">
    <div class="about-header">
      <div class="about-logo">
        <span class="logo-text">S</span>
      </div>
      <h2 class="about-title">CapellaUI</h2>
      <p class="about-subtitle">现代化的 Vue3 管理后台框架</p>
    </div>

    <div class="about-content">
      <div class="info-section">
        <div class="info-item">
          <span class="info-label">版本</span>
          <span class="info-value">v1.0.0</span>
        </div>
        <div class="info-item">
          <span class="info-label">构建时间</span>
          <span class="info-value">2025-04-20</span>
        </div>
        <div class="info-item">
          <span class="info-label">技术栈</span>
          <div class="tech-tags">
            <n-tag size="small" type="primary">Vue 3</n-tag>
            <n-tag size="small" type="success">TypeScript</n-tag>
            <n-tag size="small" type="warning">Naive UI</n-tag>
          </div>
        </div>
      </div>

      <n-divider />

      <div class="about-footer">
        <p class="copyright">© 2025 CapellaUI. All rights reserved.</p>
        <p class="license">Released under MIT License</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NTag, NDivider } from 'naive-ui'
</script>

<style scoped>
.about-modal {
  padding: 8px 0;
}

.about-header {
  text-align: center;
  margin-bottom: 24px;
}

.about-logo {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-color-hover) 100%);
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.logo-text {
  font-size: 32px;
  font-weight: 700;
  color: #fff;
}

.about-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 8px;
}

.about-subtitle {
  font-size: 14px;
  color: var(--text-color-secondary);
  margin: 0;
}

.about-content {
  background: var(--card-color);
  border-radius: 8px;
  padding: 20px;
}

.info-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.info-label {
  font-size: 14px;
  color: var(--text-color-secondary);
}

.info-value {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-color);
}

.tech-tags {
  display: flex;
  gap: 8px;
}

.about-footer {
  text-align: center;
  padding-top: 8px;
}

.copyright {
  font-size: 12px;
  color: var(--text-color-secondary);
  margin: 0 0 4px;
}

.license {
  font-size: 12px;
  color: var(--text-color-tertiary);
  margin: 0;
}
</style>
