<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import VDraggable from 'vuedraggable'
import SidebarChannelItem from './SidebarChannelItem.vue'
import { ArrowRight } from '@element-plus/icons-vue'
import type { ChannelItemData } from './SidebarChannelItem.vue'

interface Props {
  name: string
  items: ChannelItemData[]
  collapsed?: boolean
  draggable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  collapsed: false,
  draggable: true,
})

const emit = defineEmits<{
  'update:items': [items: ChannelItemData[]]
  select: [item: ChannelItemData]
  close: [item: ChannelItemData]
}>()

const localItems = ref<ChannelItemData[]>(props.items.slice())
const focusedIndex = ref(-1)
const itemRefs = ref<HTMLElement[]>([])

watch(() => props.items, (next) => {
  localItems.value = next.slice()
}, { deep: true })

function onChange() {
  emit('update:items', localItems.value)
}

const isCollapsed = ref(props.collapsed)

function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

function handleKeydown(event: KeyboardEvent) {
  if (isCollapsed.value) return
  
  const items = localItems.value
  if (items.length === 0) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      focusedIndex.value = Math.min(focusedIndex.value + 1, items.length - 1)
      focusItem(focusedIndex.value)
      break
    case 'ArrowUp':
      event.preventDefault()
      focusedIndex.value = Math.max(focusedIndex.value - 1, 0)
      focusItem(focusedIndex.value)
      break
    case 'Enter':
    case ' ':
      event.preventDefault()
      if (focusedIndex.value >= 0 && focusedIndex.value < items.length) {
        const selectedItem = items[focusedIndex.value]
        if (selectedItem) {
          emit('select', selectedItem)
        }
      }
      break
    case 'Home':
      event.preventDefault()
      focusedIndex.value = 0
      focusItem(focusedIndex.value)
      break
    case 'End':
      event.preventDefault()
      focusedIndex.value = items.length - 1
      focusItem(focusedIndex.value)
      break
  }
}

function focusItem(index: number) {
  nextTick(() => {
    const itemEl = itemRefs.value[index]
    if (itemEl) {
      itemEl.focus()
      itemEl.scrollIntoView({ block: 'nearest' })
    }
  })
}

function setItemRef(el: any, index: number) {
  if (el) {
    itemRefs.value[index] = el.$el || el
  }
}
</script>

<template>
  <div class="sidebar-category" @keydown="handleKeydown">
    <div 
      class="sidebar-category__header" 
      @click="toggleCollapse"
      @keydown.enter="toggleCollapse"
      @keydown.space.prevent="toggleCollapse"
      :tabindex="0"
      role="button"
      :aria-expanded="!isCollapsed"
    >
      <el-icon
        class="sidebar-category__arrow"
        :class="{ 'sidebar-category__arrow--collapsed': isCollapsed }"
        :size="12"
      ><ArrowRight /></el-icon>
      <span class="sidebar-category__name">{{ name }}</span>
      <span class="sidebar-category__count">{{ items.length }}</span>
    </div>

    <div class="sidebar-category__body" :class="{ 'sidebar-category__body--hidden': isCollapsed }">
      <div class="sidebar-category__inner">
        <VDraggable
          v-if="draggable"
          v-model="localItems"
          group="channels"
          item-key="id"
          handle=".channel-item__drag"
          :animation="200"
          ghost-class="channel-item--ghost"
          @change="onChange"
        >
          <template #item="{ element, index }">
            <div 
              :ref="(el) => setItemRef(el, index)"
              :tabindex="focusedIndex === index ? 0 : -1"
              @focus="focusedIndex = index"
            >
              <SidebarChannelItem
                :item="element"
                @select="emit('select', element)"
                @close="emit('close', element)"
              />
            </div>
          </template>
        </VDraggable>

        <template v-else>
          <div 
            v-for="(item, index) in items"
            :key="item.id"
            :ref="(el) => setItemRef(el, index)"
            :tabindex="focusedIndex === index ? 0 : -1"
            @focus="focusedIndex = index"
          >
            <SidebarChannelItem
              :item="item"
              @select="emit('select', item)"
              @close="emit('close', item)"
            />
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.sidebar-category {
  &__header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 12px 12px 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    cursor: pointer;
    user-select: none;

    &:hover {
      color: var(--fg);
    }
  }

  &__arrow {
    flex-shrink: 0;
    transition: transform 0.2s ease;
    color: var(--muted);

    &--collapsed {
      transform: rotate(-90deg);
    }
  }

  &__name {
    flex: 1;
  }

  &__count {
    font-size: 11px;
    opacity: 0.6;
  }

  &__body {
    display: grid;
    grid-template-rows: 1fr;
    transition: grid-template-rows 0.2s ease;
    overflow: hidden;

    &--hidden {
      grid-template-rows: 0fr;
    }
  }

  &__inner {
    overflow: hidden;
    min-height: 0;
  }
}

:global(.channel-item--ghost) {
  opacity: 0.3;
  background: var(--accent-soft);
  border-radius: var(--radius);
}
</style>
