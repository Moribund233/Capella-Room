<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  visible?: boolean
}>(), {
  visible: false,
})

const emit = defineEmits<{
  select: [emoji: string]
  close: []
}>()

const pickerRef = ref<HTMLElement | null>(null)

const EMOJIS = [
  '👍', '❤️', '😂', '😮', '😢', '😡',
  '🎉', '🔥', '💯', '✅', '❌', '⭐',
  '👀', '🙏', '💪', '🤣', '🥰', '😎',
  '🤔', '👏', '✨', '💀', '🫡', '😭',
]

function handleSelect(emoji: string) {
  emit('select', emoji)
  emit('close')
}

function onClickOutside(e: MouseEvent) {
  if (props.visible && pickerRef.value && !pickerRef.value.contains(e.target as Node)) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('click', onClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside)
})
</script>

<template>
  <transition name="emoji-fade">
    <div v-if="visible" ref="pickerRef" class="emoji-picker" @click.stop>
      <div class="emoji-picker__grid">
        <button
          v-for="emoji in EMOJIS"
          :key="emoji"
          class="emoji-picker__item"
          :title="emoji"
          @click="handleSelect(emoji)"
        >
          {{ emoji }}
        </button>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.emoji-picker {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 6px;
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e0e0e0);
  border-radius: 12px;
  padding: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  z-index: 50;
  min-width: 260px;
}

.emoji-picker__grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 2px;
}

.emoji-picker__item {
  background: none;
  border: none;
  font-size: 24px;
  padding: 6px;
  cursor: pointer;
  border-radius: 8px;
  transition: background 0.1s;
  text-align: center;
  line-height: 1;
}

.emoji-picker__item:hover {
  background: var(--message-hover, #f0f0f0);
}

.emoji-fade-enter-active,
.emoji-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.emoji-fade-enter-from,
.emoji-fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
