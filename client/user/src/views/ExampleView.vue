<template>
  <div class="example-view">
    <RouterView />
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<style scoped>
.example-view {
  min-height: 100%;
}
</style>
