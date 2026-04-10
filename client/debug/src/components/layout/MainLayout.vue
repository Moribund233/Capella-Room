<script setup lang="ts">
import Sidebar from './Sidebar.vue'
</script>

<template>
  <n-message-provider>
    <n-dialog-provider>
      <n-layout has-sider class="main-layout">
        <n-layout-sider
          :width="240"
          :collapsed-width="64"
          :native-scrollbar="false"
          class="layout-sider"
        >
          <Sidebar />
        </n-layout-sider>
        <n-layout class="layout-content">
          <n-layout-content class="content-wrapper">
            <slot />
          </n-layout-content>
        </n-layout>
      </n-layout>
    </n-dialog-provider>
  </n-message-provider>
</template>

<style scoped>
.main-layout {
  min-height: 100vh;
}

.layout-sider {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  z-index: 100;
}

.layout-content {
  margin-left: 240px;
  min-height: 100vh;
  background-color: var(--bg-primary);
}

.content-wrapper {
  min-height: 100vh;
}

@media screen and (max-width: 767px) {
  .layout-sider {
    display: none;
  }

  .layout-content {
    margin-left: 0;
  }
}
</style>
