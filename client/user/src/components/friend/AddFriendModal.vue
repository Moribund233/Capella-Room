<script setup lang="ts">
import { ref } from 'vue'
import { UserPlus, Search, X } from 'lucide-vue-next'
import { searchApi } from '@/api/search'
import { useFriendStore } from '@/stores/friend'
import UserCard from '@/components/user/UserCard.vue'
import type { UserSearchItem } from '@/types/search'

defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  done: []
}>()

const friendStore = useFriendStore()

const keyword = ref('')
const searchResults = ref<UserSearchItem[]>([])
const searching = ref(false)
const searchError = ref('')

const sending = ref(false)
const sendError = ref('')
const sendSuccess = ref(false)

let searchTimer: ReturnType<typeof setTimeout> | null = null

async function handleSearch() {
  const q = keyword.value.trim()
  if (!q) return

  searching.value = true
  searchError.value = ''
  sendSuccess.value = false

  try {
    const res = await searchApi.searchUsers({ keyword: q, limit: 10 })
    if (res.success && res.data) {
      searchResults.value = res.data.users
    } else {
      searchError.value = res.message || '搜索失败'
    }
  } catch {
    searchError.value = '搜索出错'
  } finally {
    searching.value = false
  }
}

function onKeywordInput() {
  if (searchTimer) clearTimeout(searchTimer)
  const q = keyword.value.trim()
  if (q.length < 1) {
    searchResults.value = []
    return
  }
  searchTimer = setTimeout(handleSearch, 400)
}

async function handleAddFriend(userId: string) {
  sending.value = true
  sendError.value = ''

  const ok = await friendStore.sendFriendRequest({ target_user_id: userId })
  if (ok) {
    sendSuccess.value = true
    keyword.value = ''
    searchResults.value = []
  } else {
    sendError.value = friendStore.error || '发送请求失败'
  }

  sending.value = false
}

function handleClose() {
  keyword.value = ''
  searchResults.value = []
  searchError.value = ''
  sendError.value = ''
  sendSuccess.value = false
  emit('close')
}

function handleDone() {
  emit('done')
  handleClose()
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal-container">
        <div class="modal-header">
          <h3 class="modal-title">
            <UserPlus :size="20" />
            <span>添加好友</span>
          </h3>
          <button class="modal-close" @click="handleClose">
            <X :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <!-- 搜索输入 -->
          <div class="search-box">
            <Search :size="16" class="search-box__icon" />
            <input
              v-model="keyword"
              type="text"
              class="search-box__input"
              placeholder="搜索用户名..."
              @input="onKeywordInput"
            />
          </div>

          <!-- 搜索结果 -->
          <div v-if="searching" class="search-status">搜索中...</div>
          <div v-else-if="searchError" class="search-status search-status--error">{{ searchError }}</div>
          <div v-else-if="sendSuccess" class="search-status search-status--success">好友请求已发送！</div>
          <div v-else-if="sendError" class="search-status search-status--error">{{ sendError }}</div>

          <div v-if="searchResults.length > 0" class="search-results">
            <div
              v-for="user in searchResults"
              :key="user.id"
              class="search-result-item"
            >
              <UserCard :user="user" :clickable="false" />
              <button
                class="add-btn"
                :disabled="sending"
                @click="handleAddFriend(user.id)"
              >
                {{ sending ? '发送中...' : '添加' }}
              </button>
            </div>
          </div>

          <p v-if="keyword && !searching && searchResults.length === 0 && !searchError" class="search-status">
            未找到用户
          </p>
        </div>

        <div class="modal-footer">
          <button class="btn btn--secondary" @click="handleDone">完成</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--color-mask);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  background: var(--color-white);
  border-radius: 12px;
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 8px 32px var(--color-shadow-dark);
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border, #eee);
  flex-shrink: 0;
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text, #333);
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary, #666);
  cursor: pointer;
  transition: background var(--duration-fast, 0.15s);
}

.modal-close:hover {
  background: var(--color-background, #f5f5f5);
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  flex: 1;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-box__icon {
  position: absolute;
  left: 12px;
  color: var(--color-text-tertiary, #999);
}

.search-box__input {
  width: 100%;
  padding: 10px 12px 10px 40px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 8px;
  font-size: 14px;
  outline: none;
  transition: border-color var(--duration-fast, 0.15s);
  box-sizing: border-box;
}

.search-box__input:focus {
  border-color: var(--color-primary, #2080f0);
}

.search-status {
  text-align: center;
  padding: 16px;
  color: var(--color-text-tertiary, #999);
  font-size: 13px;
}

.search-status--error {
  color: var(--color-error, #f5222d);
}

.search-status--success {
  color: var(--color-success, #52c41a);
}

.search-results {
  margin-top: 12px;
}

.search-result-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border, #f0f0f0);
}

.search-result-item:last-child {
  border-bottom: none;
}

.add-btn {
  padding: 6px 14px;
  border: 1px solid var(--color-primary, #2080f0);
  border-radius: 6px;
  background: transparent;
  color: var(--color-primary, #2080f0);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  flex-shrink: 0;
  white-space: nowrap;
}

.add-btn:hover:not(:disabled) {
  background: var(--color-primary, #2080f0);
  color: white;
}

.add-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--color-border, #eee);
  flex-shrink: 0;
}

.btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-fast, 0.15s);
  border: none;
}

.btn--secondary {
  background: var(--color-background, #f5f5f5);
  color: var(--color-text-secondary, #666);
}

.btn--secondary:hover {
  background: var(--color-background-hover, #e8e8e8);
}
</style>
