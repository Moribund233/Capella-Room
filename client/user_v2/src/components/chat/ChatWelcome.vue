<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ChatDotSquare } from '@element-plus/icons-vue'

const { t } = useI18n()
</script>

<template>
  <div class="chat-welcome">
    <div class="chat-welcome__inner">
      <div class="chat-welcome__icon">
        <el-icon :size="64">
          <ChatDotSquare />
        </el-icon>
      </div>
      <h2 class="chat-welcome__title">{{ t('chat.welcome.title') }}</h2>
      <p class="chat-welcome__desc">{{ t('chat.welcome.description') }}</p>
    </div>
  </div>
</template>

<style scoped lang="scss">
.chat-welcome {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;

  &__inner {
    text-align: center;
    max-width: 360px;
  }

  &__icon {
    display: flex;
    justify-content: center;
    margin-bottom: 24px;
    color: var(--accent);
    opacity: 0.6;
  }

  &__title {
    font-size: 20px;
    font-weight: 600;
    color: var(--fg);
    margin: 0 0 8px;
  }

  &__desc {
    font-size: 14px;
    color: var(--muted);
    margin: 0;
    line-height: 1.5;
  }
}
</style>
