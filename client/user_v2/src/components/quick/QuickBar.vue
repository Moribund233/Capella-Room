<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ArrowDown } from '@element-plus/icons-vue'
import type { QuickItem } from './types'

/**
 * 组件属性定义
 */
interface Props {
  /** Quick 项列表 */
  items: QuickItem[]
}

const props = defineProps<Props>()

/**
 * 组件事件定义
 */
const emit = defineEmits<{
  /** 点击菜单项 */
  (e: 'select', key: string, childKey?: string): void
}>()

/** 当前展开的菜单 key */
const openMenuKey = ref<string | null>(null)

/**
 * 外显的项
 */
const visibleItems = computed(() =>
  props.items.filter((item) => item.display === 'visible')
)

/**
 * 下拉菜单的项
 */
const dropdownItems = computed(() =>
  props.items.filter((item) => item.display === 'dropdown')
)

/**
 * 处理点击
 */
function handleClick(item: QuickItem): void {
  if (item.onClick) {
    item.onClick()
  }
  emit('select', item.key)
}

/**
 * 切换菜单展开
 */
function toggleMenu(key: string): void {
  openMenuKey.value = openMenuKey.value === key ? null : key
}

/**
 * 处理子菜单选择
 */
function handleChildSelect(parentKey: string, childKey: string): void {
  const item = props.items.find((i) => i.key === parentKey)
  if (item?.onSelect) {
    item.onSelect(childKey)
  }
  emit('select', parentKey, childKey)
  openMenuKey.value = null
}

/** 点击外部关闭菜单 */
function onDocumentClick(e: MouseEvent): void {
  const target = e.target as HTMLElement
  if (!target.closest('.quick-bar__menu-wrapper')) {
    openMenuKey.value = null
  }
}

onMounted(() => document.addEventListener('click', onDocumentClick))
onUnmounted(() => document.removeEventListener('click', onDocumentClick))
</script>

<template>
  <div class="quick-bar">
    <!-- 外显按钮区域 -->
    <template v-for="item in visibleItems" :key="item.key">
      <!-- 有子菜单的项（自定义面板） -->
      <div
        v-if="item.children && item.children.length > 0"
        class="quick-bar__menu-wrapper"
      >
        <button
          class="quick-bar__item"
          :class="{
            'quick-bar__item--active': item.isActive || openMenuKey === item.key,
          }"
          :title="item.label"
          @click.stop="toggleMenu(item.key)"
        >
          <el-icon :size="20">
            <component :is="item.icon" />
          </el-icon>
          <span v-if="item.badge" class="quick-bar__badge">{{ item.badge }}</span>
        </button>

        <transition name="scale">
          <div
            v-if="openMenuKey === item.key"
            class="quick-bar__panel"
          >
            <button
              v-for="child in item.children"
              :key="child.key"
              class="quick-bar__panel-item"
              :disabled="child.disabled"
              :title="child.label"
              @click.stop="handleChildSelect(item.key, child.key)"
            >
              <template v-if="child.icon">
                <span v-if="typeof child.icon === 'string'" class="quick-bar__panel-emoji">{{ child.icon }}</span>
                <el-icon v-else :size="20">
                  <component :is="child.icon" />
                </el-icon>
              </template>
              <span v-else class="quick-bar__panel-label">{{ child.label }}</span>
            </button>
          </div>
        </transition>
      </div>

      <!-- 普通按钮 -->
      <button
        v-else
        class="quick-bar__item"
        :class="{ 'quick-bar__item--active': item.isActive }"
        :title="item.label"
        @click="handleClick(item)"
      >
        <el-icon :size="20">
          <component :is="item.icon" />
        </el-icon>
        <span v-if="item.badge" class="quick-bar__badge">{{ item.badge }}</span>
      </button>
    </template>

    <!-- 下拉菜单按钮（聚合所有 dropdown 类型的按钮） -->
    <el-dropdown
      v-if="dropdownItems.length > 0"
      trigger="click"
      placement="right"
      @command="(cmd: string) => {
        const [parentKey, childKey] = cmd.split(':')
        if (childKey && parentKey) {
          handleChildSelect(parentKey, childKey)
        } else if (parentKey) {
          const item = dropdownItems.find(i => i.key === parentKey)
          if (item) handleClick(item)
        }
      }"
    >
      <button class="quick-bar__item" title="更多">
        <el-icon :size="20">
          <ArrowDown />
        </el-icon>
      </button>
      <template #dropdown>
        <el-dropdown-menu>
          <template v-for="item in dropdownItems" :key="item.key">
            <template v-if="item.children && item.children.length > 0">
              <el-dropdown-item divided disabled>
                {{ item.label }}
              </el-dropdown-item>
              <el-dropdown-item
                v-for="child in item.children"
                :key="child.key"
                :command="`${item.key}:${child.key}`"
                :disabled="child.disabled"
              >
                {{ child.label }}
              </el-dropdown-item>
            </template>

            <el-dropdown-item
              v-else
              :command="item.key"
              @click="handleClick(item)"
            >
              {{ item.label }}
            </el-dropdown-item>
          </template>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
  </div>
</template>

<style scoped lang="scss">
.quick-bar {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  align-items: center;
}

.quick-bar__item {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--muted);
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;

  &:hover {
    background: var(--message-hover);
    color: var(--fg);
  }

  &--active {
    color: var(--accent);
    background: var(--accent-light);

    &:hover {
      color: var(--accent);
      background: var(--accent-light);
    }
  }
}

.quick-bar__badge {
  position: absolute;
  top: 4px;
  right: 4px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 8px;
  background: var(--accent);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 菜单弹出面板 */
.quick-bar__menu-wrapper {
  position: relative;
}

.quick-bar__panel {
  position: absolute;
  left: 100%;
  bottom: 0;
  margin-left: 8px;
  background: var(--el-bg-color, #fff);
  border: 1px solid var(--el-border-color-light, #e4e7ed);
  border-radius: 10px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 500;
  white-space: nowrap;
}

.quick-bar__panel-item {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--muted);
  cursor: pointer;
  transition: all 0.15s ease;

  &:hover {
    background: var(--message-hover, #f0f0f0);
    color: var(--fg, #303133);
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

.quick-bar__panel-label {
  font-size: 13px;
  color: var(--fg, #303133);
}

.quick-bar__panel-emoji {
  font-size: 20px;
  line-height: 1;
}

/* 面板展开动画 */
.scale-enter-active,
.scale-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.scale-enter-from,
.scale-leave-to {
  opacity: 0;
  transform: scale(0.9);
  transform-origin: left center;
}
</style>
