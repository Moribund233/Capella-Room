<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NInput, NSelect, NSpace, NButton, NDatePicker } from 'naive-ui'
import { Search, RefreshCw, ArrowRight } from 'lucide-vue-next'
import { roomsApi, type RoomInfo } from '@/api/rooms'

/**
 * 搜索参数
 */
export interface MessageSearchParams {
  keyword: string
  roomId: string | null
  messageType: string | null
  startTime: number | null
  endTime: number | null
}

/**
 * 组件属性
 */
interface Props {
  keyword?: string
  roomId?: string | null
  messageType?: string | null
  startTime?: number | null
  endTime?: number | null
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  keyword: '',
  roomId: null,
  messageType: null,
  startTime: null,
  endTime: null,
  loading: false,
})

const emit = defineEmits<{
  search: [params: MessageSearchParams]
  reset: []
  refresh: []
}>()

/** 房间列表 */
const roomOptions = ref<{ label: string; value: string }[]>([])
const roomsLoading = ref(false)

/** 消息类型选项 */
const messageTypeOptions = [
  { label: '全部类型', value: '' },
  { label: '文本', value: 'text' },
  { label: '图片', value: 'image' },
  { label: '文件', value: 'file' },
  { label: '系统', value: 'system' },
]

/**
 * 获取房间列表
 */
const fetchRooms = async () => {
  roomsLoading.value = true
  try {
    const response = await roomsApi.getRoomList({ limit: 100 })
    if (response.success && response.data) {
      roomOptions.value = [
        { label: '全部房间', value: '' },
        ...response.data.map((room: RoomInfo) => ({
          label: room.name,
          value: room.id,
        })),
      ]
    }
  } catch (error) {
    console.error('获取房间列表失败:', error)
  } finally {
    roomsLoading.value = false
  }
}

/**
 * 处理搜索
 */
const handleSearch = () => {
  emit('search', {
    keyword: props.keyword,
    roomId: props.roomId,
    messageType: props.messageType,
    startTime: props.startTime,
    endTime: props.endTime,
  })
}

/**
 * 处理开始时间变化
 * @param value 时间戳
 */
const handleStartTimeChange = (value: number | null) => {
  // 如果开始时间晚于结束时间，清空结束时间
  if (value && props.endTime && value > props.endTime) {
    emit('search', {
      keyword: props.keyword,
      roomId: props.roomId,
      messageType: props.messageType,
      startTime: value,
      endTime: null,
    })
  } else {
    emit('search', {
      keyword: props.keyword,
      roomId: props.roomId,
      messageType: props.messageType,
      startTime: value,
      endTime: props.endTime,
    })
  }
}

/**
 * 处理结束时间变化
 * @param value 时间戳
 */
const handleEndTimeChange = (value: number | null) => {
  // 如果结束时间早于开始时间，清空开始时间
  if (value && props.startTime && value < props.startTime) {
    emit('search', {
      keyword: props.keyword,
      roomId: props.roomId,
      messageType: props.messageType,
      startTime: null,
      endTime: value,
    })
  } else {
    emit('search', {
      keyword: props.keyword,
      roomId: props.roomId,
      messageType: props.messageType,
      startTime: props.startTime,
      endTime: value,
    })
  }
}

onMounted(() => {
  fetchRooms()
})
</script>

<template>
  <NSpace align="center" wrap>
    <NInput
      :value="keyword"
      placeholder="搜索消息内容"
      clearable
      style="width: 240px"
      @update:value="(v) => emit('search', { keyword: v, roomId, messageType, startTime, endTime })"
      @keyup.enter="handleSearch"
    >
      <template #prefix>
        <Search :size="16" />
      </template>
    </NInput>

    <NSelect
      :value="roomId"
      :options="roomOptions"
      :loading="roomsLoading"
      placeholder="选择房间"
      clearable
      style="width: 180px"
      @update:value="(v) => emit('search', { keyword, roomId: v || null, messageType, startTime, endTime })"
    />

    <NSelect
      :value="messageType"
      :options="messageTypeOptions"
      placeholder="消息类型"
      clearable
      style="width: 140px"
      @update:value="(v) => emit('search', { keyword, roomId, messageType: v, startTime, endTime })"
    />

    <!-- 开始时间和结束时间分开选择 -->
    <div class="date-range-wrapper">
      <NDatePicker
        type="datetime"
        :value="startTime"
        placeholder="开始时间"
        clearable
        class="date-picker-start"
        @update:value="handleStartTimeChange"
      />
      <span class="date-range-separator">
        <ArrowRight :size="14" />
      </span>
      <NDatePicker
        type="datetime"
        :value="endTime"
        placeholder="结束时间"
        clearable
        class="date-picker-end"
        @update:value="handleEndTimeChange"
      />
    </div>

    <NButton type="primary" :loading="loading" @click="handleSearch">
      <template #icon>
        <Search :size="16" />
      </template>
      搜索
    </NButton>

    <NButton :disabled="loading" @click="emit('reset')">
      重置
    </NButton>

    <NButton quaternary :loading="loading" @click="emit('refresh')">
      <template #icon>
        <RefreshCw :size="16" />
      </template>
      刷新
    </NButton>
  </NSpace>
</template>

<style scoped>
/* 日期范围选择器包装器 */
.date-range-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 开始和结束日期选择器 */
.date-picker-start,
.date-picker-end {
  width: 180px;
}

/* 分隔符 */
.date-range-separator {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-color-3);
  flex-shrink: 0;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .date-range-wrapper {
    flex-direction: column;
    align-items: stretch;
    gap: 4px;
    width: 100%;
  }

  .date-picker-start,
  .date-picker-end {
    width: 100%;
  }

  .date-range-separator {
    align-self: center;
    transform: rotate(90deg);
    padding: 4px 0;
  }
}

@media (max-width: 480px) {
  .date-picker-start,
  .date-picker-end {
    width: 100%;
    min-width: unset;
  }
}
</style>
