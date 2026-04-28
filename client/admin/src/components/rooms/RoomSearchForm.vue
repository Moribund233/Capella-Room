<script setup lang="ts">
import { ref, watch } from 'vue'
import { NForm, NFormItem, NInput, NButton, NSpace } from 'naive-ui'
import { Search, RotateCcw, RefreshCw } from 'lucide-vue-next'

/**
 * 组件属性
 */
interface Props {
  /** 关键词 */
  keyword: string
  /** 加载状态 */
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
})

/**
 * 组件事件
 */
interface Emits {
  /** 搜索 */
  (e: 'search', values: { keyword: string }): void
  /** 重置 */
  (e: 'reset'): void
  /** 刷新 */
  (e: 'refresh'): void
}

const emit = defineEmits<Emits>()

/** 表单数据 */
const formData = ref({
  keyword: props.keyword,
})

/** 同步外部 keyword 变化 */
watch(() => props.keyword, (newVal) => {
  formData.value.keyword = newVal
})

/**
 * 处理搜索
 */
const handleSearch = () => {
  emit('search', { keyword: formData.value.keyword })
}

/**
 * 处理重置
 */
const handleReset = () => {
  formData.value.keyword = ''
  emit('reset')
}

/**
 * 处理刷新
 */
const handleRefresh = () => {
  emit('refresh')
}
</script>

<template>
  <NForm inline :show-label="false" class="room-search-form">
    <NFormItem>
      <NInput
        v-model:value="formData.keyword"
        placeholder="搜索房间名称..."
        clearable
        style="width: 280px"
        @keyup.enter="handleSearch"
      >
        <template #prefix>
          <Search :size="16" />
        </template>
      </NInput>
    </NFormItem>
    <NFormItem>
      <NSpace>
        <NButton type="primary" :loading="loading" @click="handleSearch">
          搜索
        </NButton>
        <NButton :loading="loading" @click="handleReset">
          <template #icon>
            <RotateCcw :size="16" />
          </template>
          重置
        </NButton>
        <NButton :loading="loading" @click="handleRefresh">
          <template #icon>
            <RefreshCw :size="16" />
          </template>
          刷新
        </NButton>
      </NSpace>
    </NFormItem>
  </NForm>
</template>

<style scoped>
.room-search-form {
  margin-bottom: 0;
}

@media screen and (max-width: 768px) {
  .room-search-form {
    flex-direction: column;
    align-items: flex-start;
  }

  .room-search-form :deep(.n-form-item) {
    margin-right: 0;
    width: 100%;
  }

  .room-search-form :deep(.n-input) {
    width: 100% !important;
  }
}
</style>
