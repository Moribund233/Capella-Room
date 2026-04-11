<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import {
  Search,
  UserPlus,
  Edit,
  Trash2,
  Shield,
  User,
  Mail,
  Clock,
  CheckCircle,
  XCircle,
  RefreshCw
} from 'lucide-vue-next'
import { getUsers, deleteUser, createUser, updateUser, type User as UserType } from '@/api'

const message = useMessage()
const dialog = useDialog()

// ========== 状态 ==========
const users = ref<UserType[]>([])
const loading = ref(false)
const searchQuery = ref('')
const showCreateModal = ref(false)
const showEditModal = ref(false)
const selectedUser = ref<UserType | null>(null)

// 表单数据
const newUser = ref({
  username: '',
  email: '',
  password: '',
  role: 'user' as 'user' | 'admin'
})

const editForm = ref({
  username: '',
  email: '',
  role: 'user' as 'user' | 'admin' | 'super_admin',
  status: 'active' as 'active' | 'inactive'
})

// ========== 计算属性 ==========
const filteredUsers = computed(() => {
  if (!searchQuery.value) return users.value
  const query = searchQuery.value.toLowerCase()
  return users.value.filter(
    (user) =>
      user.username.toLowerCase().includes(query) ||
      user.email.toLowerCase().includes(query)
  )
})

// ========== 表格列定义 ==========
const columns = [
  { title: '用户名', key: 'username', width: 150 },
  { title: '邮箱', key: 'email', width: 200 },
  { title: '角色', key: 'role', width: 120 },
  { title: '状态', key: 'status', width: 100 },
  { title: '创建时间', key: 'created_at', width: 150 },
  { title: '最后登录', key: 'last_login', width: 150 },
  { title: '操作', key: 'actions', width: 150, fixed: 'right' as const },
]

// ========== 数据加载 ==========
const loadUsers = async () => {
  loading.value = true
  try {
    const data = await getUsers()
    users.value = data
  } catch (error) {
    message.error('加载用户列表失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

// ========== 创建用户 ==========
const handleCreate = async () => {
  if (!newUser.value.username.trim()) {
    message.warning('请输入用户名')
    return
  }
  if (!newUser.value.email.trim()) {
    message.warning('请输入邮箱')
    return
  }
  if (!newUser.value.password.trim()) {
    message.warning('请输入密码')
    return
  }

  try {
    await createUser({
      username: newUser.value.username.trim(),
      email: newUser.value.email.trim(),
      password: newUser.value.password.trim(),
      role: newUser.value.role,
    })
    message.success('用户创建成功')
    showCreateModal.value = false
    resetCreateForm()
    loadUsers()
  } catch (error) {
    message.error('创建用户失败')
    console.error(error)
  }
}

// ========== 编辑用户 ==========
const openEditModal = (user: UserType) => {
  selectedUser.value = user
  editForm.value = {
    username: user.username,
    email: user.email,
    role: user.role,
    status: user.status,
  }
  showEditModal.value = true
}

const handleUpdate = async () => {
  if (!selectedUser.value) return
  if (!editForm.value.username.trim()) {
    message.warning('请输入用户名')
    return
  }

  try {
    await updateUser(selectedUser.value.id, {
      username: editForm.value.username.trim(),
      email: editForm.value.email.trim(),
      role: editForm.value.role,
      status: editForm.value.status,
    })
    message.success('用户更新成功')
    showEditModal.value = false
    loadUsers()
  } catch (error) {
    message.error('更新用户失败')
    console.error(error)
  }
}

// ========== 删除用户 ==========
const handleDelete = (user: UserType) => {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除用户 "${user.username}" 吗？此操作不可恢复。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await deleteUser(user.id)
        message.success('用户已删除')
        loadUsers()
      } catch (error) {
        message.error('删除用户失败')
        console.error(error)
      }
    },
  })
}

// ========== 辅助函数 ==========
const resetCreateForm = () => {
  newUser.value = {
    username: '',
    email: '',
    password: '',
    role: 'user',
  }
}

const getRoleText = (role: string) => {
  const roleMap: Record<string, string> = {
    user: '普通用户',
    admin: '管理员',
    super_admin: '超级管理员',
  }
  return roleMap[role] || role
}

const getRoleType = (role: string): 'default' | 'primary' | 'error' => {
  const typeMap: Record<string, 'default' | 'primary' | 'error'> = {
    user: 'default',
    admin: 'primary',
    super_admin: 'error',
  }
  return typeMap[role] || 'default'
}

// ========== 初始化 ==========
onMounted(() => {
  loadUsers()
})
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1 class="page-title">
        <User class="icon-lg" style="display: inline; vertical-align: middle; margin-right: 8px" />
        用户管理
      </h1>
      <p class="page-subtitle">管理系统用户，查看用户信息和状态</p>
    </div>

    <!-- 操作栏 -->
    <n-card style="margin-bottom: var(--space-lg)">
      <n-space justify="space-between" align="center">
        <n-space>
          <n-input v-model:value="searchQuery" placeholder="搜索用户..." style="width: 300px">
            <template #prefix>
              <Search class="icon-sm" />
            </template>
          </n-input>
          <n-button @click="loadUsers">
            <template #icon>
              <RefreshCw class="icon-sm" />
            </template>
            刷新
          </n-button>
        </n-space>
        <n-button type="primary" @click="showCreateModal = true">
          <template #icon>
            <UserPlus class="icon-sm" />
          </template>
          添加用户
        </n-button>
      </n-space>
    </n-card>

    <!-- 用户列表 -->
    <n-card>
      <n-data-table
        :columns="columns"
        :data="filteredUsers"
        :loading="loading"
        :bordered="false"
        :scroll-x="900"
      >
        <template #bodyCell="{ column, row }">
          <template v-if="column.key === 'username'">
            <n-space align="center">
              <n-avatar size="small" :style="{ backgroundColor: 'var(--primary)' }">
                {{ row.username.charAt(0).toUpperCase() }}
              </n-avatar>
              <span style="font-weight: 500">{{ row.username }}</span>
            </n-space>
          </template>
          <template v-if="column.key === 'email'">
            <n-space align="center">
              <Mail class="icon-sm" style="color: var(--text-muted)" />
              <span>{{ row.email }}</span>
            </n-space>
          </template>
          <template v-if="column.key === 'role'">
            <n-tag :type="getRoleType(row.role)" size="small">
              <template #icon>
                <Shield class="icon-sm" />
              </template>
              {{ getRoleText(row.role) }}
            </n-tag>
          </template>
          <template v-if="column.key === 'status'">
            <n-tag :type="row.status === 'active' ? 'success' : 'warning'" size="small">
              <template #icon>
                <component :is="row.status === 'active' ? CheckCircle : XCircle" class="icon-sm" />
              </template>
              {{ row.status === 'active' ? '活跃' : '未激活' }}
            </n-tag>
          </template>
          <template v-if="column.key === 'created_at'">
            <n-space align="center">
              <Clock class="icon-sm" style="color: var(--text-muted)" />
              <span>{{ new Date(row.created_at).toLocaleString() }}</span>
            </n-space>
          </template>
          <template v-if="column.key === 'last_login'">
            <n-space align="center">
              <Clock class="icon-sm" style="color: var(--text-muted)" />
              <span>{{ row.last_login ? new Date(row.last_login).toLocaleString() : '-' }}</span>
            </n-space>
          </template>
          <template v-if="column.key === 'actions'">
            <n-space>
              <n-button size="small" text type="primary" @click="openEditModal(row)">
                <template #icon>
                  <Edit class="icon-sm" />
                </template>
                编辑
              </n-button>
              <n-button size="small" text type="error" @click="handleDelete(row)">
                <template #icon>
                  <Trash2 class="icon-sm" />
                </template>
                删除
              </n-button>
            </n-space>
          </template>
        </template>
      </n-data-table>
    </n-card>

    <!-- 创建用户弹窗 -->
    <n-modal v-model:show="showCreateModal" title="添加用户" preset="card" style="width: 500px">
      <n-form label-placement="left" label-width="80">
        <n-form-item label="用户名" required>
          <n-input v-model:value="newUser.username" placeholder="输入用户名" />
        </n-form-item>
        <n-form-item label="邮箱" required>
          <n-input v-model:value="newUser.email" placeholder="输入邮箱" />
        </n-form-item>
        <n-form-item label="密码" required>
          <n-input v-model:value="newUser.password" type="password" placeholder="输入密码" />
        </n-form-item>
        <n-form-item label="角色">
          <n-select v-model:value="newUser.role" :options="[
            { label: '普通用户', value: 'user' },
            { label: '管理员', value: 'admin' }
          ]" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCreateModal = false">取消</n-button>
          <n-button type="primary" @click="handleCreate">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 编辑用户弹窗 -->
    <n-modal v-model:show="showEditModal" title="编辑用户" preset="card" style="width: 500px">
      <n-form label-placement="left" label-width="80">
        <n-form-item label="用户名" required>
          <n-input v-model:value="editForm.username" placeholder="输入用户名" />
        </n-form-item>
        <n-form-item label="邮箱" required>
          <n-input v-model:value="editForm.email" placeholder="输入邮箱" />
        </n-form-item>
        <n-form-item label="角色">
          <n-select v-model:value="editForm.role" :options="[
            { label: '普通用户', value: 'user' },
            { label: '管理员', value: 'admin' },
            { label: '超级管理员', value: 'super_admin' }
          ]" />
        </n-form-item>
        <n-form-item label="状态">
          <n-select v-model:value="editForm.status" :options="[
            { label: '活跃', value: 'active' },
            { label: '未激活', value: 'inactive' }
          ]" />
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
