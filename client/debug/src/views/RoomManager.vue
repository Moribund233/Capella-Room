<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import {
  Plus,
  Search,
  Edit,
  Trash2,
  Users,
  Lock,
  Globe,
  MessageSquare,
  RefreshCw,
  LogIn,
  LogOut,
} from 'lucide-vue-next'
import {
  getRooms,
  createRoom,
  updateRoom,
  deleteRoom,
  joinRoom,
  leaveRoom,
  getMyRooms,
  type Room,
} from '@/api'
import { useAuthStore } from '@/stores/auth'

const message = useMessage()
const dialog = useDialog()
const authStore = useAuthStore()

// ========== 状态 ==========
const rooms = ref<Room[]>([])
const myRooms = ref<Room[]>([])
const loading = ref(false)
const searchQuery = ref('')
const showCreateModal = ref(false)
const showEditModal = ref(false)
const selectedRoom = ref<Room | null>(null)

// 表单数据
const roomForm = ref({
  name: '',
  description: '',
  is_private: false,
})

// ========== 计算属性 ==========
const filteredRooms = computed(() => {
  if (!searchQuery.value) return rooms.value
  const query = searchQuery.value.toLowerCase()
  return rooms.value.filter(
    (room) =>
      room.name.toLowerCase().includes(query) ||
      room.description?.toLowerCase().includes(query)
  )
})

const isMyRoom = (room: Room) => {
  return (myRooms.value || []).some((r) => r.id === room.id)
}

// 判断是否可以编辑房间（房主）
const canEdit = (room: Room) => {
  const currentUserId = authStore.user?.id
  const ownerId = room.owner?.id
  console.log('[RoomManager] canEdit:', { roomId: room.id, currentUserId, ownerId, canEdit: currentUserId === ownerId })
  return currentUserId === ownerId
}

// 判断是否可以删除房间（房主或管理员）
const canDelete = (room: Room) => {
  const currentUserId = authStore.user?.id
  const ownerId = room.owner?.id
  const isAdmin = authStore.isAdmin
  const canDelete = currentUserId === ownerId || isAdmin
  console.log('[RoomManager] canDelete:', { roomId: room.id, currentUserId, ownerId, isAdmin, canDelete })
  return canDelete
}

// ========== 表格列定义 ==========
import { h } from 'vue'
import { NButton, NSpace, NTag, NEllipsis, NAvatar, NIcon } from 'naive-ui'

const columns = [
  {
    title: '房间ID',
    key: 'id',
    width: 220,
    render: (row: Room) => h(NEllipsis, { style: 'max-width: 200px' }, () => row.id)
  },
  {
    title: '房间名称',
    key: 'name',
    width: 200,
    render: (row: Room) => h(NSpace, { align: 'center' }, () => [
      h('span', { style: 'font-weight: 500' }, row.name),
      isMyRoom(row) ? h(NTag, { size: 'tiny', type: 'primary' }, () => '已加入') : null
    ])
  },
  {
    title: '描述',
    key: 'description',
    ellipsis: { tooltip: true },
    render: (row: Room) => row.description || '-'
  },
  {
    title: '类型',
    key: 'is_private',
    width: 100,
    render: (row: Room) => h(NTag, { type: row.is_private ? 'warning' : 'success', size: 'small' }, () => [
      h(NIcon, { component: row.is_private ? Lock : Globe, class: 'icon-sm' }),
      row.is_private ? ' 私有' : ' 公开'
    ])
  },
  {
    title: '成员数',
    key: 'member_count',
    width: 100,
    render: (row: Room) => h(NSpace, { align: 'center' }, () => [
      h(NIcon, { component: Users, class: 'icon-sm' }),
      String(row.member_count)
    ])
  },
  {
    title: '创建者',
    key: 'owner',
    width: 120,
    render: (row: Room) => h(NSpace, { align: 'center' }, () => [
      h(NAvatar, {
        size: 'small',
        round: true,
        src: row.owner?.avatar_url
      }, () => row.owner?.username?.charAt(0)?.toUpperCase() || '?'),
      h(NEllipsis, { style: 'max-width: 80px' }, () => row.owner?.username || 'Unknown')
    ])
  },
  {
    title: '操作',
    key: 'actions',
    width: 200,
    fixed: 'right',
    render: (row: Room) => {
      const buttons: ReturnType<typeof h>[] = []

      // 加入/离开按钮
      if (!isMyRoom(row)) {
        buttons.push(h(NButton, {
          size: 'small',
          type: 'primary',
          text: true,
          onClick: () => handleJoin(row)
        }, () => [h(NIcon, { component: LogIn, class: 'icon-sm' }), ' 加入']))
      } else {
        buttons.push(h(NButton, {
          size: 'small',
          type: 'warning',
          text: true,
          onClick: () => handleLeave(row)
        }, () => [h(NIcon, { component: LogOut, class: 'icon-sm' }), ' 离开']))
      }

      // 编辑按钮
      if (canEdit(row)) {
        buttons.push(h(NButton, {
          size: 'small',
          type: 'info',
          text: true,
          onClick: () => openEditModal(row)
        }, () => [h(NIcon, { component: Edit, class: 'icon-sm' }), ' 编辑']))
      }

      // 删除按钮
      if (canDelete(row)) {
        buttons.push(h(NButton, {
          size: 'small',
          type: 'error',
          text: true,
          onClick: () => handleDelete(row)
        }, () => [h(NIcon, { component: Trash2, class: 'icon-sm' }), ' 删除']))
      }

      return h(NSpace, {}, () => buttons)
    }
  },
]

// ========== 数据加载 ==========
const loadRooms = async () => {
  loading.value = true
  try {
    const [allRooms, userRooms] = await Promise.all([
      getRooms(),
      getMyRooms(),
    ])
    rooms.value = allRooms
    myRooms.value = userRooms
  } catch (error) {
    message.error('加载房间列表失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// ========== 创建房间 ==========
const handleCreate = async () => {
  if (!roomForm.value.name.trim()) {
    message.warning('请输入房间名称')
    return
  }

  try {
    await createRoom({
      name: roomForm.value.name.trim(),
      description: roomForm.value.description.trim() || undefined,
      is_private: roomForm.value.is_private,
    })
    message.success('房间创建成功')
    showCreateModal.value = false
    resetForm()
    loadRooms()
  } catch (error) {
    message.error('创建房间失败')
    console.error(error)
  }
}

// ========== 编辑房间 ==========
const openEditModal = (room: Room) => {
  selectedRoom.value = room
  roomForm.value = {
    name: room.name,
    description: room.description || '',
    is_private: room.is_private,
  }
  showEditModal.value = true
}

const handleUpdate = async () => {
  if (!selectedRoom.value) return
  if (!roomForm.value.name.trim()) {
    message.warning('请输入房间名称')
    return
  }

  try {
    await updateRoom(selectedRoom.value.id, {
      name: roomForm.value.name.trim(),
      description: roomForm.value.description.trim() || undefined,
      is_private: roomForm.value.is_private,
    })
    message.success('房间更新成功')
    showEditModal.value = false
    resetForm()
    loadRooms()
  } catch (error) {
    message.error('更新房间失败')
    console.error(error)
  }
}

// ========== 删除房间 ==========
const handleDelete = (room: Room) => {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除房间 "${room.name}" 吗？此操作不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await deleteRoom(room.id)
        message.success('房间已删除')
        loadRooms()
      } catch (error) {
        message.error('删除房间失败')
        console.error(error)
      }
    },
  })
}

// ========== 加入/离开房间 ==========
const handleJoin = async (room: Room) => {
  try {
    await joinRoom(room.id)
    message.success(`已加入房间 "${room.name}"`)
    loadRooms()
  } catch (error) {
    message.error('加入房间失败')
    console.error(error)
  }
}

const handleLeave = async (room: Room) => {
  dialog.warning({
    title: '确认离开',
    content: `确定要离开房间 "${room.name}" 吗？`,
    positiveText: '离开',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await leaveRoom(room.id)
        message.success(`已离开房间 "${room.name}"`)
        loadRooms()
      } catch (error) {
        message.error('离开房间失败')
        console.error(error)
      }
    },
  })
}

// ========== 辅助函数 ==========
const resetForm = () => {
  roomForm.value = {
    name: '',
    description: '',
    is_private: false,
  }
  selectedRoom.value = null
}

const openCreateModal = () => {
  resetForm()
  showCreateModal.value = true
}

// ========== 生命周期 ==========
onMounted(() => {
  loadRooms()
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <MessageSquare
          class="icon-lg"
          style="display: inline; vertical-align: middle; margin-right: 8px"
        />
        房间管理
      </h1>
      <p class="page-subtitle">管理聊天室，创建、编辑和删除房间</p>
    </div>

    <!-- 操作栏 -->
    <n-card style="margin-bottom: var(--space-lg)">
      <n-space justify="space-between" align="center">
        <n-space>
          <n-input
            v-model:value="searchQuery"
            placeholder="搜索房间..."
            style="width: 300px"
            clearable
          >
            <template #prefix>
              <Search class="icon-sm" />
            </template>
          </n-input>
          <n-button @click="loadRooms">
            <template #icon>
              <RefreshCw class="icon-sm" />
            </template>
            刷新
          </n-button>
        </n-space>
        <n-button type="primary" @click="openCreateModal">
          <template #icon>
            <Plus class="icon-sm" />
          </template>
          创建房间
        </n-button>
      </n-space>
    </n-card>

    <!-- 房间列表 -->
    <n-card>
      <n-data-table
        :columns="columns"
        :data="filteredRooms"
        :bordered="false"
        :loading="loading"
        :scroll-x="1000"
      />
    </n-card>

    <!-- 创建房间弹窗 -->
    <n-modal
      v-model:show="showCreateModal"
      title="创建新房间"
      preset="card"
      style="width: 500px"
      :mask-closable="false"
    >
      <n-form label-placement="left" label-width="80">
        <n-form-item label="房间名称" required>
          <n-input v-model:value="roomForm.name" placeholder="输入房间名称" maxlength="50" show-count />
        </n-form-item>
        <n-form-item label="描述">
          <n-input
            v-model:value="roomForm.description"
            type="textarea"
            :rows="3"
            placeholder="输入房间描述"
            maxlength="200"
            show-count
          />
        </n-form-item>
        <n-form-item label="私有房间">
          <n-switch v-model:value="roomForm.is_private">
            <template #checked>私有</template>
            <template #unchecked>公开</template>
          </n-switch>
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCreateModal = false">取消</n-button>
          <n-button type="primary" @click="handleCreate">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 编辑房间弹窗 -->
    <n-modal
      v-model:show="showEditModal"
      title="编辑房间"
      preset="card"
      style="width: 500px"
      :mask-closable="false"
    >
      <n-form label-placement="left" label-width="80">
        <n-form-item label="房间名称" required>
          <n-input v-model:value="roomForm.name" placeholder="输入房间名称" maxlength="50" show-count />
        </n-form-item>
        <n-form-item label="描述">
          <n-input
            v-model:value="roomForm.description"
            type="textarea"
            :rows="3"
            placeholder="输入房间描述"
            maxlength="200"
            show-count
          />
        </n-form-item>
        <n-form-item label="私有房间">
          <n-switch v-model:value="roomForm.is_private">
            <template #checked>私有</template>
            <template #unchecked>公开</template>
          </n-switch>
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showEditModal = false">取消</n-button>
          <n-button type="primary" @click="handleUpdate">保存</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>
