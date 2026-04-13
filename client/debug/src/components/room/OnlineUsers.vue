<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMessage } from 'naive-ui'
import { Upload, UserPlus, UserX, LogOut, Users, LogIn } from 'lucide-vue-next'
import { authManager, loadCredentialsFromFile, type TestUser, type UserCredential } from '@/utils/authUtils'

const props = defineProps<{
  roomId?: string
}>()

const emit = defineEmits<{
  (e: 'usersChanged', users: TestUser[]): void
  (e: 'selectUser', user: TestUser): void
  (e: 'joinRoom', user: TestUser): void
}>()

const message = useMessage()

// ========== 状态 ==========
const loading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
const selectedUserId = ref<string | null>(null)

// ========== 计算属性 ==========
const onlineUsers = computed<TestUser[]>(() => authManager.getAuthenticatedUsers())

// ========== 文件上传处理 ==========
const triggerFileUpload = () => {
  fileInput.value?.click()
}

const handleFileChange = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  loading.value = true
  try {
    const result = await authManager.loadAndAuthenticateFromFile(file)

    if (result.success.length > 0) {
      message.success(`成功认证 ${result.success.length} 个用户`)
      emit('usersChanged', authManager.getAuthenticatedUsers())
    }

    if (result.failed.length > 0) {
      message.warning(`${result.failed.length} 个用户认证失败`)
      console.error('认证失败的用户:', result.failed)
    }
  } catch (error) {
    message.error('加载凭证文件失败: ' + (error instanceof Error ? error.message : String(error)))
  } finally {
    loading.value = false
    // 清空文件输入
    if (fileInput.value) {
      fileInput.value.value = ''
    }
  }
}

// ========== 添加单个用户 ==========
const showAddUserModal = ref(false)
const newUserForm = ref({
  username: '',
  email: '',
  password: '',
})

const handleAddUser = async () => {
  if (!newUserForm.value.email || !newUserForm.value.password) {
    message.error('请填写邮箱和密码')
    return
  }

  loading.value = true
  try {
    const username = newUserForm.value.username || newUserForm.value.email.split('@')[0] || 'user'
    const credential: UserCredential = {
      username,
      email: newUserForm.value.email,
      password: newUserForm.value.password,
    }

    const user = await authManager.authenticateUser(credential)
    message.success(`用户 ${user.username} 认证成功`)
    emit('usersChanged', authManager.getAuthenticatedUsers())
    showAddUserModal.value = false
    newUserForm.value = { username: '', email: '', password: '' }
  } catch (error) {
    message.error('认证失败: ' + (error instanceof Error ? error.message : String(error)))
  } finally {
    loading.value = false
  }
}

// ========== 移除用户 ==========
const handleRemoveUser = (userId: string) => {
  const user = authManager.getUser(userId)
  if (user) {
    authManager.removeUser(userId)
    if (selectedUserId.value === userId) {
      selectedUserId.value = null
    }
    message.success(`用户 ${user.username} 已移除`)
    emit('usersChanged', authManager.getAuthenticatedUsers())
  }
}

// ========== 选择用户 ==========
const handleSelectUser = (user: TestUser) => {
  selectedUserId.value = user.id
  emit('selectUser', user)
}

// ========== 清空所有用户 ==========
const handleClearAll = () => {
  authManager.clearUsers()
  selectedUserId.value = null
  message.success('已清空所有在线用户')
  emit('usersChanged', [])
}

// ========== 加入房间 ==========
const handleJoinRoom = (user: TestUser) => {
  if (!props.roomId) {
    message.warning('请先选择房间')
    return
  }
  emit('joinRoom', user)
}
</script>

<template>
  <div class="online-users">
    <n-card title="在线用户" size="small">
      <template #header-extra>
        <n-space>
          <n-tag size="small" type="success">
            <template #icon>
              <Users class="icon-xs" />
            </template>
            {{ onlineUsers.length }}
          </n-tag>
        </n-space>
      </template>

      <n-spin :show="loading">
        <n-list v-if="onlineUsers.length > 0" size="small">
          <n-list-item
            v-for="user in onlineUsers"
            :key="user.id"
            :class="['user-item', { 'user-selected': selectedUserId === user.id }]"
            @click="handleSelectUser(user)"
          >
            <n-thing>
              <template #avatar>
                <n-avatar size="small" :style="{ backgroundColor: 'var(--success)' }">
                  {{ user.username.charAt(0).toUpperCase() }}
                </n-avatar>
              </template>
              <template #header>
                <n-space align="center" size="small">
                  <span>{{ user.username }}</span>
                  <n-tag v-if="user.role === 'admin' || user.role === 'super_admin'" size="tiny" type="warning">
                    {{ user.role === 'super_admin' ? '超管' : '管理员' }}
                  </n-tag>
                </n-space>
              </template>
              <template #description>
                <span class="user-email">{{ user.email }}</span>
              </template>
              <template #header-extra>
                <n-space size="small">
                  <n-button
                    v-if="roomId"
                    size="tiny"
                    type="primary"
                    @click.stop="handleJoinRoom(user)"
                  >
                    <template #icon>
                      <LogIn class="icon-xs" />
                    </template>
                    加入
                  </n-button>
                  <n-button
                    size="tiny"
                    text
                    type="error"
                    @click.stop="handleRemoveUser(user.id)"
                  >
                    <template #icon>
                      <UserX class="icon-xs" />
                    </template>
                  </n-button>
                </n-space>
              </template>
            </n-thing>
          </n-list-item>
        </n-list>

        <n-empty v-else description="暂无在线用户">
          <template #extra>
            <n-space vertical align="center">
              <n-text depth="3">点击"加载凭证"添加测试用户</n-text>
            </n-space>
          </template>
        </n-empty>
      </n-spin>

      <template #action>
        <n-space vertical>
          <n-button block type="primary" @click="triggerFileUpload">
            <template #icon>
              <Upload class="icon-sm" />
            </template>
            加载凭证文件
          </n-button>
          <n-button block @click="showAddUserModal = true">
            <template #icon>
              <UserPlus class="icon-sm" />
            </template>
            手动添加用户
          </n-button>
          <n-button
            v-if="onlineUsers.length > 0"
            block
            type="error"
            ghost
            @click="handleClearAll"
          >
            <template #icon>
              <LogOut class="icon-sm" />
            </template>
            清空所有用户
          </n-button>
        </n-space>
      </template>
    </n-card>

    <!-- 隐藏的文件输入 -->
    <input
      ref="fileInput"
      type="file"
      accept=".toml,.txt"
      style="display: none"
      @change="handleFileChange"
    />

    <!-- 手动添加用户弹窗 -->
    <n-modal
      v-model:show="showAddUserModal"
      title="添加用户"
      preset="card"
      style="width: 400px"
    >
      <n-form>
        <n-form-item label="邮箱">
          <n-input
            v-model:value="newUserForm.email"
            placeholder="user@example.com"
          />
        </n-form-item>
        <n-form-item label="密码">
          <n-input
            v-model:value="newUserForm.password"
            type="password"
            placeholder="请输入密码"
          />
        </n-form-item>
        <n-form-item label="用户名（可选）">
          <n-input
            v-model:value="newUserForm.username"
            placeholder="留空则使用邮箱前缀"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-space justify="end">
          <n-button @click="showAddUserModal = false">取消</n-button>
          <n-button type="primary" :loading="loading" @click="handleAddUser">
            添加
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.online-users {
  height: 100%;
  overflow-y: auto;
}

.user-item {
  cursor: pointer;
  transition: background-color 0.2s;
  border-radius: var(--radius-sm);
}

.user-item:hover {
  background-color: var(--bg-secondary);
}

.user-selected {
  background-color: var(--primary-light);
}

.user-email {
  font-size: 11px;
  color: var(--text-muted);
}

.icon-xs {
  width: 14px;
  height: 14px;
}

.icon-sm {
  width: 16px;
  height: 16px;
}
</style>
