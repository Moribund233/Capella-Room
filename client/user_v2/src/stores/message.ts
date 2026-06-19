import { defineStore } from 'pinia'
import { ref } from 'vue'
import { messageApi } from '@/api/message'
import { useWebSocketStore } from './websocket'
import { useAuthStore } from './auth'
import type { Message, ReplyToMessage, PinnedMessage } from '@/types/message'
import type { NewMessagePayload, MessageEditedPayload, MessageDeletedPayload, MissedMessagesPayload, ReactionAddedPayload, ReactionRemovedPayload, MessagePinnedPayload, MessageUnpinnedPayload } from '@/types/websocket'

export const useMessageStore = defineStore('message', () => {
  const messages = ref<Message[]>([])
  const cursor = ref<string | null>(null)
  const hasMore = ref(false)
  const loading = ref(false)
  const loadingMore = ref(false)
  const error = ref<string | null>(null)
  const currentRoomId = ref<string | null>(null)
  const pinnedMessages = ref<PinnedMessage[]>([])

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
      message_type: payload.message_type || 'text',
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
  async function sendMessage(roomId: string, content: string, replyTo?: string | null, messageType?: string) {
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
      message_type: messageType || 'text',
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
    const payload: Record<string, unknown> = {
      room_id: roomId,
      content: content.trim(),
      reply_to: replyTo ?? null,
    }
    if (messageType) {
      payload.message_type = messageType
    }
    wsStore.send('ChatMessage', payload)
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

    // 自动发送已读回执（非自己发送的消息）
    if (payload.sender_id !== authStore.user?.id) {
      sendReadReceipt(payload.message_id)
    }
  }

  /** 标记消息为已读（收到对方 MessageReadReceipt 时） */
  function markMessageAsRead(messageId: string) {
    const msg = messages.value.find((m) => m.id === messageId)
    if (msg) {
      msg.read = true
    }
  }

  /** 自动发送已读回执（当前房间可见的最新消息） */
  function sendReadReceiptForRoom() {
    const wsStore = useWebSocketStore()
    const authStore = useAuthStore()
    if (!wsStore.isConnected || !currentRoomId.value) return

    // 找到最后一条别人发送的消息，发送已读确认
    for (let i = messages.value.length - 1; i >= 0; i--) {
      const m = messages.value[i]
      if (m && !m.sending && !m.is_deleted && m.sender.id !== authStore.user?.id) {
        wsStore.send('MessageRead', { message_id: m.id })
        break
      }
    }
  }

  /** 发送单条消息的已读回执 */
  function sendReadReceipt(messageId: string) {
    const wsStore = useWebSocketStore()
    if (wsStore.isConnected) {
      wsStore.send('MessageRead', { message_id: messageId })
    }
  }

  /** 处理离线消息推送（来自 WS MissedMessages） */
  function addMissedMessages(payload: MissedMessagesPayload) {
    if (payload.room_id !== currentRoomId.value) return
    if (!payload.messages?.length) return

    const existingIds = new Set(messages.value.map((m) => m.id))
    const newMsgs = payload.messages
      .filter((m) => !existingIds.has(m.message_id))
      .map((m) => ({
        id: m.message_id,
        room_id: m.room_id,
        sender: { id: m.sender_id, username: m.sender_name, avatar_url: null },
        read: false,
        content: m.content,
        message_type: 'text' as const,
        reply_to: m.reply_to,
        reply_to_message: m.reply_to_message,
        is_deleted: false,
        created_at: m.created_at,
        edit_count: 0,
        edited_at: null,
      }))

    if (newMsgs.length === 0) return
    messages.value = [...newMsgs, ...messages.value]
    // 自动发送已读回执
    sendReadReceiptForRoom()
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

  /** 添加系统消息（来自 WS SystemMessage 广播） */
  function addSystemMessage(content: string) {
    if (!currentRoomId.value) return
    const sys: Message = {
      id: `sys-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      room_id: currentRoomId.value,
      sender: { id: '', username: '', avatar_url: null },
      content,
      message_type: 'text',
      reply_to: null,
      reply_to_message: null,
      is_deleted: false,
      created_at: new Date().toISOString(),
      edit_count: 0,
      edited_at: null,
      is_system: true,
    }
    messages.value.push(sys)
  }

  /** 发送添加反应 WS 消息 */
  function addReaction(messageId: string, emoji: string) {
    const wsStore = useWebSocketStore()
    wsStore.send('AddReaction', { message_id: messageId, emoji })
  }

  /** 发送移除反应 WS 消息 */
  function removeReaction(messageId: string, emoji: string) {
    const wsStore = useWebSocketStore()
    wsStore.send('RemoveReaction', { message_id: messageId, emoji })
  }

  /** 处理 ReactionAdded（广播） */
  function handleReactionAdded(payload: ReactionAddedPayload) {
    if (payload.room_id !== currentRoomId.value) return
    const msg = messages.value.find((m) => m.id === payload.message_id)
    if (!msg) return

    if (!msg.reactions) msg.reactions = []
    const existing = msg.reactions.find((r) => r.emoji === payload.emoji)
    if (existing) {
      if (!existing.users.includes(payload.user_id)) {
        existing.count++
        existing.users.push(payload.user_id)
      }
    } else {
      msg.reactions.push({
        emoji: payload.emoji,
        count: 1,
        users: [payload.user_id],
      })
    }
  }

  /** 处理 ReactionRemoved（广播） */
  function handleReactionRemoved(payload: ReactionRemovedPayload) {
    if (payload.room_id !== currentRoomId.value) return
    const msg = messages.value.find((m) => m.id === payload.message_id)
    if (!msg || !msg.reactions) return

    const existing = msg.reactions.find((r) => r.emoji === payload.emoji)
    if (!existing) return

    const idx = existing.users.indexOf(payload.user_id)
    if (idx !== -1) {
      existing.users.splice(idx, 1)
      existing.count--
    }
    if (existing.count <= 0) {
      msg.reactions = msg.reactions.filter((r) => r.emoji !== payload.emoji)
    }
  }

  /** 获取房间置顶消息 */
  async function fetchPinnedMessages(roomId: string) {
    try {
      const res = await messageApi.getRoomPinnedMessages(roomId)
      if (res.success && res.data) {
        pinnedMessages.value = res.data
      }
    } catch (err) {
      console.error('[MessageStore] fetchPinnedMessages error:', err)
    }
  }

  /** 发送置顶消息 WS */
  function pinMessage(messageId: string, roomId: string) {
    const wsStore = useWebSocketStore()
    wsStore.send('PinMessage', { message_id: messageId, room_id: roomId })
  }

  /** 发送取消置顶 WS */
  function unpinMessage(messageId: string, roomId: string) {
    const wsStore = useWebSocketStore()
    wsStore.send('UnpinMessage', { message_id: messageId, room_id: roomId })
  }

  /** 处理 MessagePinned（广播） */
  function handleMessagePinned(payload: MessagePinnedPayload) {
    if (payload.room_id !== currentRoomId.value) return
    if (pinnedMessages.value.some((m) => m.message_id === payload.message_id)) return
    pinnedMessages.value.unshift({
      id: '',
      message_id: payload.message_id,
      room_id: payload.room_id,
      pinned_by: payload.pinned_by,
      content: payload.content_preview,
      sender_name: payload.pinned_by_name,
      created_at: payload.pinned_at,
    })
  }

  /** 处理 MessageUnpinned（广播） */
  function handleMessageUnpinned(payload: MessageUnpinnedPayload) {
    if (payload.room_id !== currentRoomId.value) return
    pinnedMessages.value = pinnedMessages.value.filter(
      (m) => m.message_id !== payload.message_id,
    )
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
    pinnedMessages.value = []
  }

  function $reset() {
    messages.value = []
    cursor.value = null
    hasMore.value = false
    loading.value = false
    loadingMore.value = false
    error.value = null
    currentRoomId.value = null
    pinnedMessages.value = []
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
    markMessageAsRead,
    sendReadReceiptForRoom,
    sendMessage,
    confirmMessage,
    failMessage,
    addSystemMessage,
    addIncomingMessage,
    addMissedMessages,
    handleMessageEdited,
    handleMessageDeleted,
    editMessage,
    deleteMessage,
    addReaction,
    removeReaction,
    handleReactionAdded,
    handleReactionRemoved,
    pinnedMessages,
    fetchPinnedMessages,
    pinMessage,
    unpinMessage,
    handleMessagePinned,
    handleMessageUnpinned,
    switchRoom,
    $reset,
  }
})
