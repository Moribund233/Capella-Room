<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { fetchTrendingGifs, searchGifs, type GifResult } from '@/api/gif'

const { t } = useI18n()

const props = withDefaults(defineProps<{
  visible?: boolean
}>(), {
  visible: false,
})

const emit = defineEmits<{
  select: [url: string]
  close: []
}>()

const pickerRef = ref<HTMLElement | null>(null)
const searchQuery = ref('')
const gifs = ref<GifResult[]>([])
const loading = ref(false)
const error = ref('')

let searchTimer: ReturnType<typeof setTimeout> | null = null

async function loadTrending() {
  loading.value = true
  error.value = ''
  try {
    gifs.value = await fetchTrendingGifs()
  } catch {
    error.value = t('common.error')
  } finally {
    loading.value = false
  }
}

async function doSearch(query: string) {
  if (!query.trim()) {
    loadTrending()
    return
  }
  loading.value = true
  error.value = ''
  try {
    gifs.value = await searchGifs(query.trim())
  } catch {
    error.value = t('common.error')
  } finally {
    loading.value = false
  }
}

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => doSearch(searchQuery.value), 400)
}

function handleSelect(gif: GifResult) {
  emit('select', gif.url)
  emit('close')
}

function onClickOutside(e: MouseEvent) {
  if (props.visible && pickerRef.value && !pickerRef.value.contains(e.target as Node)) {
    emit('close')
  }
}

watch(() => props.visible, (val) => {
  if (val) {
    searchQuery.value = ''
    loadTrending()
  }
})

onMounted(() => {
  document.addEventListener('click', onClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside)
  if (searchTimer) clearTimeout(searchTimer)
})
</script>

<template>
  <transition name="gif-fade">
    <div v-if="visible" ref="pickerRef" class="gif-picker" @click.stop>
      <div class="gif-picker__search">
        <input
          v-model="searchQuery"
          :placeholder="t('chat.searchGif')"
          @input="onSearchInput"
        />
      </div>
      <div class="gif-picker__body">
        <div v-if="loading" class="gif-picker__loading">
          <span class="gif-picker__spinner" />
        </div>
        <div v-else-if="error" class="gif-picker__error">{{ error }}</div>
        <div v-else-if="gifs.length === 0" class="gif-picker__empty">
          {{ t('chat.noGifResults') }}
        </div>
        <div v-else class="gif-picker__grid">
          <button
            v-for="gif in gifs"
            :key="gif.id"
            class="gif-picker__item"
            :title="gif.title"
            @click="handleSelect(gif)"
          >
            <img :src="gif.previewUrl" :alt="gif.title" loading="lazy" />
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.gif-picker {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 6px;
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e0e0e0);
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  z-index: 50;
  width: 320px;
  overflow: hidden;
}

.gif-picker__search {
  padding: 8px;
  border-bottom: 1px solid var(--border, #e0e0e0);
}

.gif-picker__search input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border, #e0e0e0);
  border-radius: 8px;
  background: var(--bg, #f5f5f5);
  color: var(--fg, #333);
  font: inherit;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}

.gif-picker__search input:focus {
  border-color: var(--accent, #409eff);
}

.gif-picker__body {
  max-height: 300px;
  overflow-y: auto;
}

.gif-picker__grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 4px;
  padding: 8px;
}

.gif-picker__item {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  border-radius: 6px;
  overflow: hidden;
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--message-hover, #f0f0f0);
}

.gif-picker__item:hover {
  opacity: 0.85;
}

.gif-picker__item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.gif-picker__loading,
.gif-picker__error,
.gif-picker__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px 16px;
  font-size: 13px;
  color: var(--muted, #999);
}

.gif-picker__spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border, #e0e0e0);
  border-top-color: var(--accent, #409eff);
  border-radius: 50%;
  animation: gif-spin 0.6s linear infinite;
}

@keyframes gif-spin {
  to { transform: rotate(360deg); }
}

.gif-fade-enter-active,
.gif-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.gif-fade-enter-from,
.gif-fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
