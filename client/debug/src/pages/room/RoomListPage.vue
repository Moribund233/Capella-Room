<template>
  <div class="room-list-page">
    <n-card title="房间列表">
      <template #header-extra>
        <n-button type="primary" @click="showCreateModal = true">
          创建房间
        </n-button>
      </template>

      <n-spin :show="loading">
        <n-empty v-if="rooms.length === 0" description="暂无房间" />
        <n-list v-else>
          <n-list-item v-for="room in rooms" :key="room.id">
            <n-thing
              :title="room.name"
              :description="getRoomLastMessage(room.id)?.content || room.description || '暂无消息'"
            >
              <template #avatar>
                <n-avatar>
                  <n-icon><MessageSquare /></n-icon>
                </n-avatar>
              </template>
              <template #header-extra>
                <n-tag size="small" :type="room.is_private ? 'warning' : 'success'">
                  {{ room.is_private ? '私密' : '公开' }}
                </n-tag>
              </template>
              <template #footer>
                <n-space>
                  <n-text depth="3">
                    成员: {{ room.member_count }}/{{ room.max_members }}
                  </n-text>
                  <n-text depth="3">
                    创建者: {{ room.owner.username }}
                  </n-text>
                  <n-text v-if="getRoomLastMessage(room.id)" depth="3" type="success">
                    最新消息: {{ getRoomLastMessage(room.id)?.sender_name }}
                  </n-text>
                </n-space>
              </template>
              <template #action>
                <n-button type="primary" @click="enterRoom(room.id)">
                  进入房间
                </n-button>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>
      </n-spin>
    </n-card>

    <!-- 创建房间模态框 -->
    <n-modal
      v-model:show="showCreateModal"
      title="创建房间"
      preset="card"
      style="width: 400px"
    >
      <n-form :model="createForm" label-placement="top">
        <n-form-item label="房间名称" required>
          <n-input v-model:value="createForm.name" placeholder="请输入房间名称" />
        </n-form-item>
        <n-form-item label="房间描述">
          <n-input
            v-model:value="createForm.description"
            type="textarea"
            placeholder="请输入房间描述"
          />
        </n-form-item>
        <n-form-item label="是否私密">
          <n-switch v-model:value="createForm.is_private" />
        </n-form-item>
        <n-form-item label="最大成员数">
          <n-input-number v-model:value="createForm.max_members" :min="2" :max="100" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCreateModal = false">取消</n-button>
          <n-button type="primary" :loading="creating" @click="handleCreateRoom">
            创建
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import { MessageSquare } from 'lucide-vue-next'
import {
  NCard,
  NButton,
  NSpin,
  NEmpty,
  NList,
  NListItem,
  NThing,
  NAvatar,
  NIcon,
  NTag,
  NSpace,
  NText,
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSwitch,
  NInputNumber,
} from 'naive-ui'
import { getRooms, createRoom as createRoomApi } from '@/api/room'
import { useWebSocketStore } from '@/store/websocket'
import type { Room } from '@/types/api'
import type { MessagePreview } from '@/types/websocket'
import { storeToRefs } from 'pinia'

const router = useRouter()
const message = useMessage()
const wsStore = useWebSocketStore()
const { roomMessageSummaries } = storeToRefs(wsStore)

const rooms = ref<Room[]>([])
const loading = ref(false)
const showCreateModal = ref(false)
const creating = ref(false)
let isMounted = true

// 获取房间的最后消息预览
function getRoomLastMessage(roomId: string): MessagePreview | undefined {
  return roomMessageSummaries.value.get(roomId)?.last_message
}

const createForm = ref({
  name: '',
  description: '',
  is_private: false,
  max_members: 50,
})

async function fetchRooms() {
  loading.value = true
  try {
    const response = await getRooms()
    if (isMounted) {
      rooms.value = response.items
    }
  } catch (error) {
    console.error('获取房间列表失败:', error)
    if (isMounted) {
      message.error('获取房间列表失败')
    }
  } finally {
    if (isMounted) {
      loading.value = false
    }
  }
}

async function handleCreateRoom() {
  if (!createForm.value.name.trim()) {
    message.warning('请输入房间名称')
    return
  }

  creating.value = true
  try {
    await createRoomApi({
      name: createForm.value.name,
      description: createForm.value.description,
      is_private: createForm.value.is_private,
      max_members: createForm.value.max_members,
    })
    message.success('房间创建成功')
    showCreateModal.value = false
    createForm.value = {
      name: '',
      description: '',
      is_private: false,
      max_members: 50,
    }
    await fetchRooms()
  } catch (error) {
    console.error('创建房间失败:', error)
    message.error('创建房间失败')
  } finally {
    creating.value = false
  }
}

function enterRoom(roomId: string) {
  router.push(`/room/chat/${roomId}`)
}

onMounted(() => {
  fetchRooms()
})

onUnmounted(() => {
  isMounted = false
})
</script>

<style scoped>
.room-list-page {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
}
</style>
