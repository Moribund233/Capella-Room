<script setup lang="ts">
import { ref, watch } from 'vue'
import VDraggable from 'vuedraggable'
import SidebarChannelItem from './SidebarChannelItem.vue'
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
</script>

<template>
  <div class="sidebar-category">
    <div class="sidebar-category__header" @click="toggleCollapse">
      <svg
        class="sidebar-category__arrow"
        :class="{ 'sidebar-category__arrow--collapsed': isCollapsed }"
        viewBox="0 0 24 24"
        fill="currentColor"
        width="12"
        height="12"
      >
        <path d="M8 5l8 7-8 7" />
      </svg>
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
          <template #item="{ element }">
            <SidebarChannelItem
              :item="element"
              @select="emit('select', element)"
              @close="emit('close', element)"
            />
          </template>
        </VDraggable>

        <template v-else>
          <SidebarChannelItem
            v-for="item in items"
            :key="item.id"
            :item="item"
            @select="emit('select', item)"
            @close="emit('close', item)"
          />
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
