<script setup lang="ts">
import { ArrowRight, ArrowDown } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import { ref } from 'vue'
import { useSettingsStore } from '@/stores/settings'

const { t, locale } = useI18n()
const settingsStore = useSettingsStore()

// 语言选项
const languages = [
  { code: 'zh', label: '中文' },
  { code: 'en', label: 'English' },
  { code: 'ja', label: '日本語' },
]

// 当前语言
const currentLang = ref(locale.value)

// 语言下拉菜单显示状态
const langMenuVisible = ref(false)

/**
 * 切换语言
 * @param code - 语言代码
 */
function switchLanguage(code: string) {
  locale.value = code
  currentLang.value = code
  localStorage.setItem('locale', code)

  // 同步更新 settings store 中的语言设置
  const newLanguage = code === 'zh' ? 'zh-CN' : code === 'ja' ? 'ja-JP' : 'en-US'
  settingsStore.updateLocaleSettings({
    ...settingsStore.localeSettings,
    language: newLanguage,
  })

  langMenuVisible.value = false
}
</script>

<template>
  <div class="landing-page">
    <!-- 顶部导航 -->
    <header class="topnav">
      <div class="container topnav-inner">
        <span class="logo">
          <img src="/favicon.svg" alt="" class="logo-mark" />
          {{ t('common.appName') }}
        </span>
        <nav>
          <a href="#features">{{ t('nav.features') }}</a>
          <a href="#stats">{{ t('nav.community') }}</a>
          
          <!-- 语言切换 -->
          <div class="lang-switcher" @mouseenter="langMenuVisible = true" @mouseleave="langMenuVisible = false">
            <button class="lang-btn">
              {{ languages.find(l => l.code === currentLang)?.label }}
              <el-icon class="lang-icon" :class="{ 'is-open': langMenuVisible }"><ArrowDown /></el-icon>
            </button>
            <transition name="lang-menu">
              <div v-show="langMenuVisible" class="lang-menu">
                <button
                  v-for="lang in languages"
                  :key="lang.code"
                  class="lang-option"
                  :class="{ active: currentLang === lang.code }"
                  @click="switchLanguage(lang.code)"
                >
                  {{ lang.label }}
                </button>
              </div>
            </transition>
          </div>
          
          <router-link to="/login">{{ t('nav.signIn') }}</router-link>
          <router-link to="/login" class="btn btn-primary" style="padding: 8px 20px;">{{ t('nav.getStarted') }}</router-link>
        </nav>
      </div>
    </header>

    <main>
      <!-- Hero 区域 -->
      <section class="section hero-section">
        <div class="container hero-split">
          <div class="hero-content">
            <p class="eyebrow">{{ t('landing.hero.eyebrow') }}</p>
            <h1 class="h1">{{ t('landing.hero.title') }}</h1>
            <p class="lead">{{ t('landing.hero.description') }}</p>
            <div class="hero-actions">
              <router-link to="/login" class="btn btn-primary">
                {{ t('landing.hero.startFree') }}
                <el-icon class="btn-icon"><ArrowRight /></el-icon>
              </router-link>
              <a href="#features" class="btn btn-secondary">{{ t('landing.hero.seeFeatures') }}</a>
            </div>
          </div>
          <div class="hero-visual">
            <div class="hero-chat-bubbles">
              <div class="hero-bubble">Hey everyone! 👋 Excited for the launch tonight!</div>
              <div class="hero-bubble">The new design update is looking incredible 🔥</div>
              <div class="hero-bubble">Can't wait to share what we've been building!</div>
            </div>
          </div>
        </div>
      </section>

      <!-- 功能特性 -->
      <section class="section" id="features">
        <div class="container features-container">
          <div class="features-header">
            <p class="eyebrow">{{ t('landing.features.eyebrow') }}</p>
            <h2 class="h2">{{ t('landing.features.title') }}</h2>
          </div>
          <div class="grid-3">
            <el-card class="feature-card" shadow="never">
              <div class="feature-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
              </div>
              <h3 class="h3">{{ t('landing.features.channels.title') }}</h3>
              <p class="feature-desc">{{ t('landing.features.channels.description') }}</p>
            </el-card>
            <el-card class="feature-card" shadow="never">
              <div class="feature-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17 12a5 5 0 01-5 5M7 12a5 5 0 015-5"/>
                  <path d="M12 17a5 5 0 005-5M12 7a5 5 0 00-5 5"/>
                </svg>
              </div>
              <h3 class="h3">{{ t('landing.features.threads.title') }}</h3>
              <p class="feature-desc">{{ t('landing.features.threads.description') }}</p>
            </el-card>
            <el-card class="feature-card" shadow="never">
              <div class="feature-icon">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10"/>
                  <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
                  <line x1="9" y1="9" x2="9.01" y2="9"/>
                  <line x1="15" y1="9" x2="15.01" y2="9"/>
                </svg>
              </div>
              <h3 class="h3">{{ t('landing.features.expressions.title') }}</h3>
              <p class="feature-desc">{{ t('landing.features.expressions.description') }}</p>
            </el-card>
          </div>
        </div>
      </section>

      <!-- 统计数据 -->
      <section class="section stats-section" id="stats">
        <div class="container">
          <p class="eyebrow" style="text-align: center; margin-bottom: 40px;">{{ t('landing.stats.eyebrow') }}</p>
          <div class="stat-row">
            <div class="stat">
              <div class="stat-num">12k+</div>
              <p class="stat-label">{{ t('landing.stats.communities') }}</p>
            </div>
            <div class="stat">
              <div class="stat-num">2.4M</div>
              <p class="stat-label">{{ t('landing.stats.messages') }}</p>
            </div>
            <div class="stat">
              <div class="stat-num">98%</div>
              <p class="stat-label">{{ t('landing.stats.satisfaction') }}</p>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA 区域 -->
      <section class="section cta-section">
        <div class="container cta-content">
          <h2 class="h2">{{ t('landing.cta.title') }}</h2>
          <p class="lead" style="margin: 16px auto 32px;">{{ t('landing.cta.description') }}</p>
          <div class="cta-actions">
            <router-link to="/login" class="btn btn-primary">{{ t('landing.cta.getStarted') }}</router-link>
            <router-link to="/app" class="btn btn-secondary">
              {{ t('landing.cta.viewDemo') }}
              <el-icon class="btn-icon"><ArrowRight /></el-icon>
            </router-link>
          </div>
        </div>
      </section>
    </main>

    <!-- 页脚 -->
    <footer class="pagefoot">
      <div class="container pagefoot-inner">
        <span class="copyright">{{ t('landing.footer.copyright') }}</span>
        <div class="pagefoot-socials">
          <a href="#">Twitter</a>
          <a href="#">Discord</a>
          <a href="#">GitHub</a>
        </div>
      </div>
    </footer>
  </div>
</template>

<style scoped lang="scss">
.landing-page {
  background: var(--bg);
  color: var(--fg);
  min-height: 100vh;
}

.container {
  max-width: 1120px;
  margin-inline: auto;
  padding-inline: 32px;
}

.section {
  padding-block: clamp(60px, 10vw, 120px);
}

// 顶部导航
.topnav {
  position: sticky;
  top: 0;
  z-index: 10;
  background: color-mix(in oklch, var(--bg) 92%, transparent);
  backdrop-filter: blur(16px);
  border-bottom: 1px solid var(--border);
}

.topnav-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-block: 14px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: var(--font-display);
  font-size: 19px;
  font-weight: 600;
}

.logo-mark {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  filter: var(--logo-filter);
  transition: filter 0.2s ease;
}

.topnav nav {
  display: flex;
  gap: 28px;
  align-items: center;
}

.topnav nav a {
  font-size: 14px;
  color: var(--muted);
  transition: color 0.15s;
  text-decoration: none;

  &:hover {
    color: var(--fg);
  }
}

.topnav nav a.btn-primary {
  color: #fff;

  &:hover {
    color: #fff;
  }
}

// 移动端适配
@media (max-width: 768px) {
  .topnav nav a:not(.btn-primary) {
    display: none;
  }

  .topnav nav {
    gap: 12px;
  }

  .container {
    padding-inline: 16px;
  }

  .topnav-inner {
    padding-block: 10px;
  }
}

// 语言切换器
.lang-switcher {
  position: relative;
}

.lang-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--muted);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s ease;

  &:hover {
    color: var(--fg);
    border-color: var(--fg);
  }
}

.lang-icon {
  font-size: 12px;
  transition: transform 0.2s ease;

  &.is-open {
    transform: rotate(180deg);
  }
}

.lang-menu {
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 4px;
  min-width: 100px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
}

.lang-option {
  display: block;
  width: 100%;
  padding: 8px 12px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--muted);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  transition: all 0.15s ease;

  &:hover {
    background: var(--fg-soft);
    color: var(--fg);
  }

  &.active {
    color: var(--accent);
    font-weight: 500;
  }
}

// 语言菜单动画
.lang-menu-enter-active,
.lang-menu-leave-active {
  transition: all 0.2s ease;
}

.lang-menu-enter-from,
.lang-menu-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-8px);
}

// 按钮样式
.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border-radius: var(--radius-full);
  border: 1px solid transparent;
  font-size: 15px;
  font-weight: 500;
  transition: transform 0.05s ease, background 0.15s ease, border-color 0.15s;
  text-decoration: none;
  cursor: pointer;

  &:active {
    transform: translateY(1px);
  }
}

.btn-primary {
  background: var(--accent);
  color: #fff;

  &:hover {
    background: color-mix(in oklch, var(--accent) 88%, black);
  }
}

.btn-secondary {
  background: transparent;
  color: var(--fg);
  border-color: var(--border);

  &:hover {
    border-color: var(--fg);
  }
}

.btn-icon {
  transition: transform 0.15s;
}

.btn:hover .btn-icon {
  transform: translateX(3px);
}

// 排版
.h1 {
  font-family: var(--font-display);
  font-size: clamp(40px, 6vw, 72px);
  line-height: 1.04;
  letter-spacing: -0.025em;
  margin: 0;
}

.h2 {
  font-family: var(--font-display);
  font-size: clamp(30px, 4vw, 48px);
  line-height: 1.1;
  letter-spacing: -0.02em;
  margin: 0;
}

.h3 {
  font-size: 22px;
  font-weight: 600;
  line-height: 1.3;
  margin: 0;
}

.lead {
  font-size: 19px;
  line-height: 1.5;
  color: var(--muted);
  max-width: 60ch;
  margin: 16px 0 0;
}

.eyebrow {
  font-family: var(--font-mono);
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--accent);
  margin: 0 0 16px;
}

// Hero 区域
.hero-section {
  padding-top: 80px;
}

.hero-split {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 60px;
  align-items: center;
}

@media (max-width: 840px) {
  .hero-split {
    grid-template-columns: 1fr;
  }
}

.hero-actions {
  margin-top: 32px;
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.hero-visual {
  background: linear-gradient(135deg, color-mix(in oklch, var(--accent) 12%, var(--surface)), color-mix(in oklch, var(--accent-pink) 8%, var(--surface)));
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  aspect-ratio: 4/3;
  display: grid;
  place-items: center;
  position: relative;
  overflow: hidden;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    background: radial-gradient(circle at 40% 50%, color-mix(in oklch, var(--accent) 15%, transparent) 0%, transparent 70%);
  }
}

.hero-chat-bubbles {
  position: relative;
  z-index: 1;
  width: 80%;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hero-bubble {
  padding: 14px 18px;
  border-radius: 16px;
  font-size: 14px;
  max-width: 75%;
  animation: float 3s ease-in-out infinite;

  &:nth-child(1) {
    background: var(--accent);
    color: #fff;
    align-self: flex-start;
    animation-delay: 0s;
  }

  &:nth-child(2) {
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--fg);
    align-self: flex-end;
    animation-delay: 0.5s;
  }

  &:nth-child(3) {
    background: var(--accent-soft);
    color: var(--fg);
    align-self: flex-start;
    animation-delay: 1s;
  }
}

@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-6px);
  }
}

// 功能特性
.features-container {
  display: flex;
  flex-direction: column;
  gap: 48px;
}

.features-header {
  max-width: 40ch;
}

.grid-3 {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 28px;
}

@media (max-width: 840px) {
  .grid-3 {
    grid-template-columns: 1fr;
  }
}

.feature-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 32px;

  :deep(.el-card__body) {
    padding: 0;
  }
}

.feature-icon {
  width: 44px;
  height: 44px;
  background: var(--accent-soft);
  border-radius: 12px;
  display: grid;
  place-items: center;
  color: var(--accent);
  margin-bottom: 16px;

  svg {
    width: 22px;
    height: 22px;
  }
}

.feature-desc {
  color: var(--muted);
  font-size: 15px;
  margin: 8px 0 0;
}

// 统计数据
.stats-section {
  background: color-mix(in oklch, var(--accent) 3%, var(--bg));
}

.stat-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 40px;
  text-align: center;
}

@media (max-width: 640px) {
  .stat-row {
    grid-template-columns: 1fr;
  }
}

.stat-num {
  font-family: var(--font-display);
  font-size: clamp(48px, 7vw, 80px);
  line-height: 0.95;
  font-weight: 700;
  background: linear-gradient(135deg, var(--accent), var(--accent-pink));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.stat-label {
  color: var(--muted);
  font-size: 15px;
  margin-top: 8px;
}

// CTA 区域
.cta-section {
  text-align: center;
  position: relative;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    background: radial-gradient(ellipse 60% 50% at 50% 50%, color-mix(in oklch, var(--accent) 8%, transparent), transparent);
    pointer-events: none;
  }
}

.cta-content {
  position: relative;
  z-index: 1;
  max-width: 560px;
  margin: 0 auto;
}

.cta-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

// 页脚
.pagefoot {
  padding-block: 48px;
  border-top: 1px solid var(--border);
}

.pagefoot-inner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.copyright {
  font-size: 13px;
  color: var(--muted);
}

.pagefoot-socials {
  display: flex;
  gap: 16px;

  a {
    color: var(--muted);
    font-size: 13px;
    text-decoration: none;

    &:hover {
      color: var(--fg);
    }
  }
}
</style>
