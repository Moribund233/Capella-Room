<script setup lang="ts">
import { onMounted } from 'vue'

/**
 * 初始化主题
 * 添加 dark 类到 html 元素以启用暗黑模式
 */
onMounted(() => {
  document.documentElement.classList.add('dark')
})
</script>

<template>
  <router-view />
</template>

<style>
/* 全局样式重置 */
*,
*::before,
*::after {
  box-sizing: border-box;
}

html {
  scroll-behavior: smooth;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  padding: 0;
  font-family: var(--wave-font-body);
  -webkit-font-smoothing: antialiased;
}

/* 图片和 SVG 默认样式 */
img,
svg {
  display: block;
  max-width: 100%;
}

/* 链接默认样式 */
a {
  color: inherit;
  text-decoration: none;
}

/* 段落文本 */
p {
  text-wrap: pretty;
}

/* 标题文本 */
h1,
h2,
h3,
h4 {
  text-wrap: balance;
}
</style>
