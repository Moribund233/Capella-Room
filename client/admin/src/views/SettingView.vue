<template>
  <div class="setting-view">
    <RouterView />
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style scoped>
.setting-view {
  min-height: 100%;
}
</style>
