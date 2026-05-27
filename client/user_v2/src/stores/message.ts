import { defineStore } from 'pinia'
import { ref } from 'vue'
import { messageApi } from '@/api/message'
import { useWebSocketStore } from './websocket'
import { useAuthStore } from './auth'
import type { Message, ReplyToMessage } from '@/types/message'
import type { NewMessagePayload, MessageEditedPayload, MessageDeletedPayload } from '@/types/websocket'

export const useMessageStore = defineStore('message', () => {
  const messages = ref<Message[]>([])
  const cursor = ref<string | null>(null)
  const hasMore = ref(false)
  const loading = ref(false)
  const loadingMore = ref(false)
  const error = ref<string | null>(null)
  const currentRoomId = ref<string | null>(null)

  /** 从 WS NewMessagePayload 转换为 Message */
  function toMessage(payload: NewMessagePayload): Message {
    // 转换 reply_to_message（后端格式: sender_id/sender_name → 前端格式: sender.id/sender.username）
    let replyToMessage: ReplyToMessage | null = null
    if (payload.reply_to_message) {
      const rt = payload.reply_to_message as unknown as Record<string, unknown>
      replyToMessage = {
        id: rt.id as string,
        sender: {
          id: rt.sender_id as string,
          username: rt.sender_name as string,
          avatar_url: null,
        },
        content: rt.content as string,
        created_at: rt.created_at as string,
      }
    }

    return {
      id: payload.message_id,
      room_id: payload.room_id,
      sender: {
        id: payload.sender_id,
        username: payload.sender_name,
        avatar_url: null,
      },
      content: payload.content,
      message_type: 'text',
      reply_to: payload.reply_to,
      reply_to_message: replyToMessage,
      is_deleted: false,
      created_at: payload.created_at,
      edit_count: 0,
      edited_at: null,
    }
  }

  /** 获取消息历史（首次加载） */
  async function fetchMessages(roomId: string) {
    if (loading.value) return
    loading.value = true
    error.value = null
    currentRoomId.value = roomId

    try {
      const res = await messageApi.getRoomMessages(roomId, { limit: 50 })
      if (res.success && res.data) {
        // 后端返回 ORDER BY created_at DESC（最新在前），反向为从旧到新
        const msgs = res.data.messages.reverse()
        messages.value = msgs
        cursor.value = msgs[0]?.id ?? null
        hasMore.value = res.data.has_more
      }
    } catch (err) {
      error.value = '获取消息失败'
      console.error('[MessageStore] fetchMessages error:', err)
    } finally {
      loading.value = false
    }
  }

  /** 加载更多历史消息 */
  async function fetchMore() {
    if (loadingMore.value || !hasMore.value || !cursor.value || !currentRoomId.value) return
    loadingMore.value = true

    try {
      const res = await messageApi.getRoomMessages(currentRoomId.value, {
        before: cursor.value,
        limit: 50,
      })
      if (res.success && res.data) {
        const older = res.data.messages.reverse()
        messages.value = [...older, ...messages.value]
        cursor.value = older[0]?.id ?? null
        hasMore.value = res.data.has_more
      }
    } catch (err) {
      console.error('[MessageStore] fetchMore error:', err)
    } finally {
      loadingMore.value = false
    }
  }

  /** 发送消息（乐观更新） */
  async function sendMessage(roomId: string, content: string, replyTo?: string | null) {
    if (!content.trim()) return

    const wsStore = useWebSocketStore()
    const authStore = useAuthStore()

    // 乐观消息
    const tempId = `temp-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
    const optimistic: Message = {
      id: tempId,
      room_id: roomId,
      sender: {
        id: authStore.user?.id ?? '',
        username: authStore.user?.username ?? '',
        avatar_url: authStore.user?.avatar_url ?? null,
      },
      content: content.trim(),
      message_type: 'text',
      reply_to: replyTo ?? null,
      reply_to_message: null,
      is_deleted: false,
      created_at: new Date().toISOString(),
      edit_count: 0,
      edited_at: null,
      sending: true,
    }

    messages.value.push(optimistic)

    // 发送 WS 消息
    wsStore.send('ChatMessage', {
      room_id: roomId,
      content: content.trim(),
      reply_to: replyTo ?? null,
    })
  }

  /** 标记某条消息为发送成功（根据临时 id 替换） */
  function confirmMessage(tempId: string, realId: string) {
    const msg = messages.value.find((m) => m.id === tempId)
    if (msg) {
      msg.id = realId
      msg.sending = false
      msg.error = false
    }
  }

  /** 标记某条消息为发送失败 */
  function failMessage(tempId: string) {
    const msg = messages.value.find((m) => m.id === tempId)
    if (msg) {
      msg.sending = false
      msg.error = true
    }
  }

  /** 处理收到的新消息（来自 WS） */
  function addIncomingMessage(payload: NewMessagePayload) {
    if (payload.room_id !== currentRoomId.value) return

    // 查找并替换对应的乐观消息
    const authStore = useAuthStore()
    if (payload.sender_id === authStore.user?.id) {
      const optimisticIndex = messages.value.findIndex(
        (m) => m.sending && m.content === payload.content,
      )
      if (optimisticIndex !== -1) {
        // 替换为完整的消息对象（包含 reply_to_message）
        messages.value[optimisticIndex] = {
          ...toMessage(payload),
          sending: false,
          error: false,
        }
        return
      }
    }

    // 过滤重复
    if (messages.value.some((m) => m.id === payload.message_id)) return

    messages.value.push(toMessage(payload))
  }

  /** 处理消息编辑（来自 WS） */
  function handleMessageEdited(payload: MessageEditedPayload) {
    const index = messages.value.findIndex((m) => m.id === payload.message_id)
    if (index !== -1) {
      const msg = messages.value[index]!
      messages.value[index] = {
        ...msg,
        content: payload.new_content,
        edit_count: msg.edit_count + 1,
        edited_at: payload.edited_at,
      }
    }
  }

  /** 处理消息删除（来自 WS） */
  function handleMessageDeleted(payload: MessageDeletedPayload) {
    const index = messages.value.findIndex((m) => m.id === payload.message_id)
    if (index !== -1) {
      const msg = messages.value[index]!
      messages.value[index] = {
        ...msg,
        is_deleted: true,
        content: '此消息已被删除',
      }
    }
  }

  /** 编辑消息 */
  function editMessage(messageId: string, newContent: string) {
    const wsStore = useWebSocketStore()
    wsStore.send('EditMessage', {
      message_id: messageId,
      new_content: newContent.trim(),
    })
  }

  /** 删除消息 */
  function deleteMessage(messageId: string) {
    const wsStore = useWebSocketStore()
    wsStore.send('DeleteMessage', {
      message_id: messageId,
    })
  }

  /** 切换到另一个房间 */
  function switchRoom(roomId: string) {
    if (currentRoomId.value === roomId) return
    currentRoomId.value = roomId
    messages.value = []
    cursor.value = null
    hasMore.value = false
    loading.value = false
    loadingMore.value = false
    error.value = null
  }

  function $reset() {
    messages.value = []
    cursor.value = null
    hasMore.value = false
    loading.value = false
    loadingMore.value = false
    error.value = null
    currentRoomId.value = null
  }

  return {
    messages,
    cursor,
    hasMore,
    loading,
    loadingMore,
    error,
    currentRoomId,
    fetchMessages,
    fetchMore,
    sendMessage,
    confirmMessage,
    failMessage,
    addIncomingMessage,
    handleMessageEdited,
    handleMessageDeleted,
    editMessage,
    deleteMessage,
    switchRoom,
    $reset,
  }
})
