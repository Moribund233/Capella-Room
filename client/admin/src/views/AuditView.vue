<template>
  <div class="audit-view">
    <RouterView />
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style scoped>
.audit-view {
  min-height: 100%;
}
</style>
