<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoomStore } from '@/stores/room'
import type { CreateRoomData } from '@/types/room'

const props = defineProps<{
  onCreated?: (roomId: string) => void
  onCancel?: () => void
}>()

const { t } = useI18n()
const roomStore = useRoomStore()

const form = reactive<CreateRoomData>({
  name: '',
  description: '',
  is_private: false,
  max_members: 100,
})

const submitting = ref(false)
const nameError = ref('')

/**
 * 验证房间名称
 * @returns 是否通过验证
 */
function validateName(): boolean {
  const n = form.name.trim()
  if (!n) {
    nameError.value = t('room.nameRequired')
    return false
  }
  if (n.length > 50) {
    nameError.value = t('room.nameMaxLength')
    return false
  }
  nameError.value = ''
  return true
}

/**
 * 提交创建房间
 */
async function handleSubmit(): Promise<void> {
  if (!validateName()) return
  submitting.value = true

  try {
    const room = await roomStore.createRoom({
      name: form.name.trim(),
      description: form.description?.trim() || undefined,
      is_private: form.is_private,
      max_members: form.max_members || undefined,
    })

    if (room) {
      props.onCreated?.(room.id)
    }
  } finally {
    submitting.value = false
  }
}

/**
 * 取消创建
 */
function handleCancel(): void {
  props.onCancel?.()
}
</script>

<template>
  <div class="create-room-modal">
    <el-form label-position="top" class="create-room-form">
      <!-- 房间名称 -->
      <el-form-item :label="t('room.nameLabel')" :error="nameError">
        <el-input
          v-model="form.name"
          :placeholder="t('room.namePlaceholder')"
          maxlength="50"
          show-word-limit
          clearable
          @input="nameError = ''"
        />
      </el-form-item>

      <!-- 房间描述 -->
      <el-form-item :label="t('room.descriptionLabel')">
        <el-input
          v-model="form.description"
          type="textarea"
          :rows="3"
          :placeholder="t('room.descriptionPlaceholder')"
          maxlength="200"
          show-word-limit
          clearable
        />
      </el-form-item>

      <!-- 私密房间开关 -->
      <el-form-item>
        <div class="privacy-row">
          <el-switch v-model="form.is_private" />
          <div class="privacy-info">
            <span class="privacy-label">{{ t('room.privateLabel') }}</span>
            <p class="privacy-hint">{{ t('room.privateHint') }}</p>
          </div>
        </div>
      </el-form-item>

      <!-- 最大成员数 -->
      <el-form-item :label="t('room.maxMembersLabel')">
        <el-slider v-model="form.max_members" :min="2" :max="1000" :step="10" show-input />
      </el-form-item>
    </el-form>

    <!-- 底部按钮 -->
    <div class="modal-footer">
      <el-button @click="handleCancel">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="handleSubmit">
        {{ t('common.create') }}
      </el-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.create-room-modal {
  padding: 8px 4px;
}

.create-room-form {
  :deep(.el-form-item__label) {
    font-weight: 500;
    padding-bottom: 4px;
  }

  :deep(.el-input__wrapper),
  :deep(.el-textarea__inner) {
    border-radius: 8px;
  }
}

.privacy-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.privacy-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.privacy-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.privacy-hint {
  margin: 0;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.4;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}
</style>
