<script setup lang="ts">
import { ref } from 'vue'
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

const searchQuery = ref('')
const showCreateModal = ref(false)
const selectedUser = ref<any>(null)

const users = ref([
  {
    id: '1',
    username: 'admin',
    email: 'admin@seredeli.com',
    role: 'superadmin',
    status: 'active',
    created_at: '2024-01-01',
    last_login: '2024-03-10 10:30:00'
  },
  {
    id: '2',
    username: 'user_123',
    email: 'user123@example.com',
    role: 'user',
    status: 'active',
    created_at: '2024-01-15',
    last_login: '2024-03-09 15:20:00'
  },
  {
    id: '3',
    username: 'test_user',
    email: 'test@example.com',
    role: 'user',
    status: 'inactive',
    created_at: '2024-02-01',
    last_login: '2024-02-15 09:10:00'
  }
])

const newUser = ref({
  username: '',
  email: '',
  password: '',
  role: 'user'
})

const columns = [
  { title: '用户名', key: 'username' },
  { title: '邮箱', key: 'email' },
  { title: '角色', key: 'role' },
  { title: '状态', key: 'status' },
  { title: '最后登录', key: 'last_login' },
  { title: '操作', key: 'actions' }
]

const createUser = () => {
  users.value.push({
    id: String(users.value.length + 1),
    username: newUser.value.username,
    email: newUser.value.email,
    role: newUser.value.role,
    status: 'active',
    created_at: new Date().toISOString().split('T')[0] || '',
    last_login: '-'
  })
  showCreateModal.value = false
  newUser.value = { username: '', email: '', password: '', role: 'user' }
}

const deleteUser = (user: any) => {
  const index = users.value.findIndex((u) => u.id === user.id)
  if (index > -1) {
    users.value.splice(index, 1)
  }
}

const viewUserDetails = (user: any) => {
  selectedUser.value = user
}
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
          <n-button>
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
      <n-data-table :columns="columns" :data="users" :bordered="false">
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
            <n-tag :type="row.role === 'superadmin' ? 'error' : 'default'" size="small">
              <template #icon>
                <Shield class="icon-sm" />
              </template>
              {{ row.role === 'superadmin' ? '超级管理员' : '普通用户' }}
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
          <template v-if="column.key === 'last_login'">
            <n-space align="center">
              <Clock class="icon-sm" style="color: var(--text-muted)" />
              <span>{{ row.last_login }}</span>
            </n-space>
          </template>
          <template v-if="column.key === 'actions'">
            <n-space>
              <n-button size="small" text type="primary" @click="viewUserDetails(row)">
                <template #icon>
                  <Edit class="icon-sm" />
                </template>
                编辑
              </n-button>
              <n-button size="small" text type="error" @click="deleteUser(row)">
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
          <n-select
            v-model:value="newUser.role"
            :options="[
              { label: '普通用户', value: 'user' },
              { label: '管理员', value: 'admin' }
            ]"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showCreateModal = false">取消</n-button>
          <n-button type="primary" @click="createUser">添加</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 用户详情弹窗 -->
    <n-modal
      v-model:show="selectedUser"
      :title="selectedUser?.username"
      preset="card"
      style="width: 500px"
    >
      <n-descriptions v-if="selectedUser" :columns="2" bordered>
        <n-descriptions-item label="ID">{{ selectedUser.id }}</n-descriptions-item>
        <n-descriptions-item label="用户名">{{ selectedUser.username }}</n-descriptions-item>
        <n-descriptions-item label="邮箱" :span="2">{{ selectedUser.email }}</n-descriptions-item>
        <n-descriptions-item label="角色">
          <n-tag :type="selectedUser.role === 'superadmin' ? 'error' : 'default'">
            {{ selectedUser.role === 'superadmin' ? '超级管理员' : '普通用户' }}
          </n-tag>
        </n-descriptions-item>
        <n-descriptions-item label="状态">
          <n-tag :type="selectedUser.status === 'active' ? 'success' : 'warning'">
            {{ selectedUser.status === 'active' ? '活跃' : '未激活' }}
          </n-tag>
        </n-descriptions-item>
        <n-descriptions-item label="创建时间">{{ selectedUser.created_at }}</n-descriptions-item>
        <n-descriptions-item label="最后登录">{{ selectedUser.last_login }}</n-descriptions-item>
      </n-descriptions>
    </n-modal>
  </div>
</template>
