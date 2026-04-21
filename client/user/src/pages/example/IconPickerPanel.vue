<template>
  <div class="icon-picker-panel">
    <h2>Lucide Vue 图标库</h2>
    <p>本项目集成 Lucide Vue Next，提供 1000+ 个高质量 SVG 图标，支持搜索、分页和主题自适应。</p>

    <!-- 图标选择器展示 -->
    <n-card title="图标选择器" class="picker-card">
      <icon-picker :selected-icon="selectedIcon" @select="handleSelect" />
    </n-card>

    <!-- 选中图标预览 -->
    <n-card v-if="selectedIcon" title="选中预览" class="preview-card">
      <div class="preview-content">
        <div class="preview-icon">
          <component :is="getIconComponent(selectedIcon)" :size="48" />
        </div>
        <div class="preview-info">
          <n-statistic label="图标名称" :value="selectedIcon" />
          <n-code :code="iconUsageCode" language="html" />
        </div>
      </div>
    </n-card>

    <!-- 使用说明 -->
    <n-card title="使用方式" class="usage-card">
      <n-list>
        <n-list-item>
          <n-thing title="1. 安装依赖">
            <n-code code="pnpm add lucide-vue-next" language="bash" />
          </n-thing>
        </n-list-item>
        <n-list-item>
          <n-thing title="2. 导入图标">
            <n-code :code="importCode" language="typescript" />
          </n-thing>
        </n-list-item>
        <n-list-item>
          <n-thing title="3. 在模板中使用">
            <n-code :code="templateCode" language="html" />
          </n-thing>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- 特性说明 -->
    <n-card title="特性" class="feature-card">
      <n-grid :cols="3" :x-gap="16" :y-gap="16">
        <n-grid-item>
          <n-card size="small">
            <n-space align="center">
              <n-icon :component="Layers" :size="24" />
              <n-text strong>1000+ 图标</n-text>
            </n-space>
            <n-text depth="3">覆盖常用场景，持续更新</n-text>
          </n-card>
        </n-grid-item>
        <n-grid-item>
          <n-card size="small">
            <n-space align="center">
              <n-icon :component="Maximize" :size="24" />
              <n-text strong>SVG 矢量</n-text>
            </n-space>
            <n-text depth="3">无损缩放，高清显示</n-text>
          </n-card>
        </n-grid-item>
        <n-grid-item>
          <n-card size="small">
            <n-space align="center">
              <n-icon :component="Palette" :size="24" />
              <n-text strong>主题自适应</n-text>
            </n-space>
            <n-text depth="3">自动跟随明暗主题变色</n-text>
          </n-card>
        </n-grid-item>
      </n-grid>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  NCard,
  NStatistic,
  NCode,
  NList,
  NListItem,
  NThing,
  NGrid,
  NGridItem,
  NIcon,
  NSpace,
  NText,
} from 'naive-ui'
import { Layers, Maximize, Palette } from 'lucide-vue-next'
import * as LucideIcons from 'lucide-vue-next'
import type { FunctionalComponent } from 'vue'
import type { LucideProps } from 'lucide-vue-next'
import { IconPicker } from '@/components/common'

/**
 * 当前选中的图标名称
 */
const selectedIcon = ref('')

/**
 * 图标使用代码示例
 */
const iconUsageCode = computed(() => `<${selectedIcon.value} :size="24" />`)

/**
 * 导入代码示例
 */
const importCode = `import { Home, Settings, User } from 'lucide-vue-next'`

/**
 * 模板代码示例
 */
const templateCode = `<template>
  <Home :size="24" />
  <Settings :size="24" />
  <User :size="24" />
</template>`

/**
 * 获取图标组件
 * @param iconName 图标名称
 * @returns 图标组件
 */
function getIconComponent(iconName: string): FunctionalComponent<LucideProps> {
  return (LucideIcons as unknown as Record<string, FunctionalComponent<LucideProps>>)[iconName]
    || LucideIcons.Circle
}

/**
 * 处理图标选择
 * @param iconName 选中的图标名称
 */
function handleSelect(iconName: string): void {
  selectedIcon.value = iconName
}
</script>

<style scoped>
.icon-picker-panel {
  display: flex;
  flex-direction: column;
  padding: 16px;
  min-height: 0;
}

.icon-picker-panel h2 {
  margin-bottom: 8px;
  color: var(--text-primary);
  flex-shrink: 0;
}

.icon-picker-panel>p {
  margin-bottom: 16px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.picker-card,
.preview-card,
.usage-card,
.feature-card {
  margin-bottom: 24px;
  flex-shrink: 0;
}

.preview-content {
  display: flex;
  align-items: center;
  gap: 24px;
}

.preview-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 96px;
  height: 96px;
  border-radius: 12px;
  background: var(--bg-base);
  border: 1px solid var(--border-color-base);
  color: var(--color-primary);
  flex-shrink: 0;
}

.preview-info {
  flex: 1;
  min-width: 0;
}

@media (max-width: 768px) {
  .icon-picker-panel {
    padding: 12px;
  }

  .preview-content {
    flex-direction: column;
    align-items: flex-start;
  }

  .picker-card,
  .preview-card,
  .usage-card,
  .feature-card {
    margin-bottom: 16px;
  }
}
</style>
