<script setup lang="ts">
import { ref, computed } from 'vue'
import { Camera, Loader2 } from 'lucide-vue-next'

const props = defineProps<{
  avatarUrl: string | null | undefined
  username: string | undefined
}>()

const emit = defineEmits<{
  upload: [file: File]
}>()

const isHovering = ref(false)
const isUploading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const displayInitial = computed(() => {
  return props.username?.charAt(0).toUpperCase() || '?'
})

const avatarStyle = computed(() => {
  if (props.avatarUrl) {
    return { backgroundImage: `url(${props.avatarUrl})` }
  }
  return {}
})

function handleClick() {
  fileInput.value?.click()
}

function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    emit('upload', file)
  }
  // 重置 input 以便可以再次选择同一文件
  target.value = ''
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  isHovering.value = true
}

function handleDragLeave(event: DragEvent) {
  event.preventDefault()
  isHovering.value = false
}

function handleDrop(event: DragEvent) {
  event.preventDefault()
  isHovering.value = false
  const file = event.dataTransfer?.files[0]
  if (file && file.type.startsWith('image/')) {
    emit('upload', file)
  }
}
</script>

<template>
  <div
    class="avatar-upload"
    :class="{ 'avatar-upload--hovering': isHovering, 'avatar-upload--has-image': avatarUrl }"
    @click="handleClick"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <div
      class="avatar-upload__image"
      :style="avatarStyle"
    >
      <span v-if="!avatarUrl" class="avatar-upload__initial">{{ displayInitial }}</span>
    </div>

    <div class="avatar-upload__overlay">
      <Loader2 v-if="isUploading" class="avatar-upload__icon avatar-upload__icon--spin" />
      <Camera v-else class="avatar-upload__icon" />
      <span class="avatar-upload__text">{{ isUploading ? '上传中...' : '更换头像' }}</span>
    </div>

    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      class="avatar-upload__input"
      @change="handleFileChange"
    />
  </div>
</template>

<style scoped>
.avatar-upload {
  position: relative;
  width: 100px;
  height: 100px;
  border-radius: var(--radius-full);
  cursor: pointer;
  overflow: hidden;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-md);
}

.avatar-upload:hover,
.avatar-upload--hovering {
  transform: scale(1.05);
  box-shadow: var(--shadow-lg);
}

.avatar-upload__image {
  width: 100%;
  height: 100%;
  background-color: var(--color-primary);
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.avatar-upload__initial {
  font-size: 36px;
  font-weight: 600;
  color: var(--color-white);
  user-select: none;
}

.avatar-upload__overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.avatar-upload:hover .avatar-upload__overlay,
.avatar-upload--hovering .avatar-upload__overlay {
  opacity: 1;
}

.avatar-upload__icon {
  width: 24px;
  height: 24px;
  color: var(--color-white);
}

.avatar-upload__icon--spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.avatar-upload__text {
  font-size: var(--font-size-small);
  color: var(--color-white);
  font-weight: 500;
}

.avatar-upload__input {
  display: none;
}

/* 响应式 */
@media (max-width: 768px) {
  .avatar-upload {
    width: 80px;
    height: 80px;
  }

  .avatar-upload__initial {
    font-size: 28px;
  }
}
</style>
