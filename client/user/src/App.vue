<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { NConfigProvider, NMessageProvider, zhCN, dateZhCN } from 'naive-ui'
import { themeOverrides } from '@/styles/native-ui-overrides'
import { AuthLayout, MainLayout } from '@/layouts'

const route = useRoute()

const layout = computed(() => {
  if (route.name === 'login' || route.name === 'register') {
    return AuthLayout
  }
  return MainLayout
})
</script>

<template>
  <NConfigProvider :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <component :is="layout">
        <router-view />
      </component>
    </NMessageProvider>
  </NConfigProvider>
</template>
