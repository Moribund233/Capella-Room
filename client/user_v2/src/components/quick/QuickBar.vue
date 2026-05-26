<script setup lang="ts">
import { computed } from 'vue'
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
 * @param item - Quick项
 */
function handleClick(item: QuickItem): void {
  if (item.onClick) {
    item.onClick()
  }
  emit('select', item.key)
}

/**
 * 处理子菜单选择
 * @param parentKey - 父项key
 * @param childKey - 子项key
 */
function handleChildSelect(parentKey: string, childKey: string): void {
  const item = props.items.find((i) => i.key === parentKey)
  if (item?.onSelect) {
    item.onSelect(childKey)
  }
  emit('select', parentKey, childKey)
}
</script>

<template>
  <div class="quick-bar">
    <!-- 外显按钮区域 -->
    <template v-for="item in visibleItems" :key="item.key">
      <!-- 有子菜单的项 -->
      <el-dropdown
        v-if="item.children && item.children.length > 0"
        trigger="click"
        placement="right"
        @command="(cmd: string) => handleChildSelect(item.key, cmd)"
      >
        <button
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
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item
              v-for="child in item.children"
              :key="child.key"
              :command="child.key"
              :disabled="child.disabled"
            >
              {{ child.label }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

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
            <!-- 有子菜单的项 -->
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

            <!-- 普通项 -->
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
  color: var(--wave-muted);
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;

  &:hover {
    background: var(--wave-message-hover);
    color: var(--wave-fg);
  }

  &--active {
    color: var(--wave-accent);
    background: var(--wave-accent-light);

    &:hover {
      color: var(--wave-accent);
      background: var(--wave-accent-light);
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
  background: var(--wave-accent);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
