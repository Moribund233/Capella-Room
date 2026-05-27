import { ref, computed } from 'vue'
import { useMessageStore } from '@/stores/message'
import { useAuthStore } from '@/stores/auth'
import type { Message, ReplyToMessage } from '@/types/message'

/**
 * 消息操作组合式函数
 * 管理消息回复、编辑、删除等高级功能
 */
export function useMessageActions(roomId: string) {
  const messageStore = useMessageStore()
  const authStore = useAuthStore()

  // 回复状态
  const replyingTo = ref<ReplyToMessage | null>(null)

  // 编辑状态
  const editingMessage = ref<{ id: string; content: string } | null>(null)

  // 正在输入的用户
  const typingUsers = ref<Array<{ id: string; username: string }>>([])

  // 搜索状态
  const showSearch = ref(false)

  // 当前用户发送的消息
  const isOwnMessage = (message: Message): boolean => {
    return message.sender.id === authStore.user?.id
  }

  // 开始回复消息
  function startReply(message: Message) {
    if (message.is_deleted) return
    replyingTo.value = {
      id: message.id,
      sender: message.sender,
      content: message.content,
      created_at: message.created_at,
    }
    editingMessage.value = null
  }

  // 取消回复
  function cancelReply() {
    replyingTo.value = null
  }

  // 开始编辑消息
  function startEdit(message: Message) {
    if (message.is_deleted || !isOwnMessage(message)) return
    editingMessage.value = {
      id: message.id,
      content: message.content,
    }
    replyingTo.value = null
  }

  // 取消编辑
  function cancelEdit() {
    editingMessage.value = null
  }

  // 保存编辑
  function saveEdit(messageId: string, newContent: string) {
    if (!newContent.trim()) return
    messageStore.editMessage(messageId, newContent)
    editingMessage.value = null
  }

  // 删除消息
  function deleteMessage(message: Message) {
    if (message.is_deleted || !isOwnMessage(message)) return
    messageStore.deleteMessage(message.id)
  }

  // 发送消息（带回复）
  function sendMessage(content: string) {
    const replyToId = replyingTo.value?.id ?? null
    messageStore.sendMessage(roomId, content, replyToId)
    replyingTo.value = null
  }

  // 打开搜索
  function openSearch() {
    showSearch.value = true
  }

  // 关闭搜索
  function closeSearch() {
    showSearch.value = false
  }

  // 跳转到消息
  function jumpToMessage(messageId: string) {
    // TODO: 实现滚动到指定消息的功能
    console.log('[useMessageActions] jumpToMessage:', messageId)
  }

  // 添加正在输入的用户
  function addTypingUser(userId: string, username: string) {
    if (!typingUsers.value.some((u) => u.id === userId)) {
      typingUsers.value.push({ id: userId, username })
    }
  }

  // 移除正在输入的用户
  function removeTypingUser(userId: string) {
    typingUsers.value = typingUsers.value.filter((u) => u.id !== userId)
  }

  return {
    // 状态
    replyingTo,
    editingMessage,
    typingUsers: computed(() => typingUsers.value),
    showSearch,

    // 方法
    isOwnMessage,
    startReply,
    cancelReply,
    startEdit,
    cancelEdit,
    saveEdit,
    deleteMessage,
    sendMessage,
    openSearch,
    closeSearch,
    jumpToMessage,
    addTypingUser,
    removeTypingUser,
  }
}
