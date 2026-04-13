/**
 * 消息发送测试工具（优化版）
 * 使用多用户 WebSocket 连接发送消息
 * 无需处理连接逻辑，认证后自动连接
 */

import { h } from 'vue'
import { MessageSquare, Send, Shuffle, Zap, Users } from 'lucide-vue-next'
import { useMultiUserWebSocketStore } from '@/stores/multiUserWebSocket'
import type {
  ToolDefinition,
  ToolContext,
  ToolResult,
  ToolMenuItem,
} from './types'

/**
 * 随机消息内容池
 */
const RANDOM_MESSAGES = [
  '大家好！',
  '今天天气不错',
  '有人在线吗？',
  '测试消息',
  '哈哈',
  '收到',
  '明白了',
  '谢谢分享',
  '很有意思',
  '继续加油',
  '不错不错',
  '学到了',
  '期待更新',
  '这个功能好用',
  '有什么新消息吗？',
]

/**
 * 获取随机消息
 */
function getRandomMessage(): string {
  const index = Math.floor(Math.random() * RANDOM_MESSAGES.length)
  return RANDOM_MESSAGES[index] ?? '测试消息'
}

/**
 * 延迟函数
 */
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * 单条消息发送菜单项
 */
const singleMessageMenuItem: ToolMenuItem = {
  key: 'single',
  label: '单条消息',
  icon: () => h(Send, { size: 14 }),
  description: '以指定用户身份发送单条消息',
  params: [
    {
      key: 'userIndex',
      label: '用户序号',
      type: 'number',
      defaultValue: 0,
      min: 0,
      max: 100,
      step: 1,
      description: '选择房间成员列表中的第几个用户（从0开始，仅包含有凭证的测试用户）',
    },
    {
      key: 'message',
      label: '消息内容',
      type: 'string',
      defaultValue: '测试消息',
      description: '要发送的消息内容',
    },
  ],
}

/**
 * 随机消息发送菜单项
 */
const randomMessageMenuItem: ToolMenuItem = {
  key: 'random',
  label: '随机消息',
  icon: () => h(Shuffle, { size: 14 }),
  description: '随机选择房间成员发送随机消息',
  params: [
    {
      key: 'count',
      label: '发送数量',
      type: 'number',
      defaultValue: 5,
      min: 1,
      max: 50,
      step: 1,
      description: '要发送的消息数量',
    },
    {
      key: 'interval',
      label: '发送间隔(ms)',
      type: 'number',
      defaultValue: 1000,
      min: 100,
      max: 10000,
      step: 100,
      description: '每条消息之间的间隔时间（毫秒）',
    },
  ],
}

/**
 * 批量消息发送菜单项
 */
const burstMessageMenuItem: ToolMenuItem = {
  key: 'burst',
  label: '批量发送',
  icon: () => h(Zap, { size: 14 }),
  description: '房间中所有有凭证的测试用户同时发送消息',
  params: [
    {
      key: 'messageCount',
      label: '每人发送条数',
      type: 'number',
      defaultValue: 3,
      min: 1,
      max: 20,
      step: 1,
      description: '每个用户发送的消息数量',
    },
    {
      key: 'interval',
      label: '间隔时间(ms)',
      type: 'number',
      defaultValue: 500,
      min: 0,
      max: 5000,
      step: 100,
      description: '消息之间的间隔时间（毫秒）',
    },
    {
      key: 'useRandomMessage',
      label: '使用随机消息',
      type: 'boolean',
      defaultValue: true,
      description: '是否使用随机消息内容',
    },
  ],
}

/**
 * 消息测试工具定义
 */
export const messageTestTool: ToolDefinition = {
  key: 'messageTest',
  label: '消息发送测试',
  icon: () => h(MessageSquare, { size: 16 }),
  description: '操控房间成员发送消息',
  menuItems: [singleMessageMenuItem, randomMessageMenuItem, burstMessageMenuItem],
}

/**
 * 获取有测试凭证的房间成员
 */
function getMembersWithTestUser(context: ToolContext) {
  return context.roomMembers.filter(m => m.testUser)
}

/**
 * 发送单条消息（简化版）
 */
async function sendSingleMessage(
  context: ToolContext,
  params: Record<string, unknown>
): Promise<ToolResult> {
  const membersWithTestUser = getMembersWithTestUser(context)
  const userIndex = params.userIndex as number
  const message = params.message as string
  const wsStore = useMultiUserWebSocketStore()

  if (membersWithTestUser.length === 0) {
    return { success: false, message: '房间中没有可操控的测试用户' }
  }

  if (userIndex < 0 || userIndex >= membersWithTestUser.length) {
    return { success: false, message: `用户序号 ${userIndex} 超出范围（0-${membersWithTestUser.length - 1}）` }
  }

  const member = membersWithTestUser[userIndex]
  if (!member || !member.testUser) {
    return { success: false, message: '未找到指定用户' }
  }

  // 检查用户是否已在线（认证时自动连接）
  if (!wsStore.isUserConnected(member.testUser.id)) {
    return { success: false, message: `用户 ${member.username} 未在线，请先完成认证` }
  }

  // 确保已加入房间
  if (!wsStore.isUserInRoom(member.testUser.id, context.roomId)) {
    wsStore.joinRoom(member.testUser.id, context.roomId)
    await delay(300)

    // 再次检查
    if (!wsStore.isUserInRoom(member.testUser.id, context.roomId)) {
      return { success: false, message: `用户 ${member.username} 加入房间失败，无法发送消息` }
    }
  }

  // 发送消息
  const success = wsStore.sendChatMessage(member.testUser.id, context.roomId, message)

  if (success) {
    return { success: true, message: `用户 ${member.username} 发送消息成功`, data: { user: member, message } }
  } else {
    return { success: false, message: `用户 ${member.username} 发送消息失败` }
  }
}

/**
 * 发送随机消息（简化版）
 */
async function sendRandomMessages(
  context: ToolContext,
  params: Record<string, unknown>
): Promise<ToolResult> {
  const membersWithTestUser = getMembersWithTestUser(context)
  const count = params.count as number
  const interval = params.interval as number
  const wsStore = useMultiUserWebSocketStore()

  if (membersWithTestUser.length === 0) {
    return { success: false, message: '房间中没有可操控的测试用户' }
  }

  // 检查在线状态
  const onlineUsers = membersWithTestUser.filter(m => m.testUser && wsStore.isUserConnected(m.testUser.id))
  if (onlineUsers.length === 0) {
    return { success: false, message: '没有用户在线，请先完成认证' }
  }

  // 确保所有在线用户已加入房间
  for (const member of onlineUsers) {
    if (member.testUser && !wsStore.isUserInRoom(member.testUser.id, context.roomId)) {
      wsStore.joinRoom(member.testUser.id, context.roomId)
    }
  }
  await delay(500)

  // 获取已加入房间的用户
  const usersInRoom = onlineUsers.filter(
    m => m.testUser && wsStore.isUserInRoom(m.testUser.id, context.roomId)
  )

  if (usersInRoom.length === 0) {
    return { success: false, message: '没有用户成功加入房间，无法发送消息' }
  }

  const results: { user: string; message: string; success: boolean }[] = []

  for (let i = 0; i < count; i++) {
    const randomUserIndex = Math.floor(Math.random() * usersInRoom.length)
    const member = usersInRoom[randomUserIndex]
    const messageContent = getRandomMessage()

    if (!member || !member.testUser) {
      results.push({ user: 'unknown', message: messageContent, success: false })
      continue
    }

    const success = wsStore.sendChatMessage(member.testUser.id, context.roomId, messageContent)
    results.push({ user: member.username, message: messageContent, success })

    if (i < count - 1 && interval > 0) {
      await delay(interval)
    }
  }

  const successCount = results.filter(r => r.success).length
  return {
    success: successCount > 0,
    message: `随机消息发送完成: ${successCount}/${count} 成功`,
    data: { results },
  }
}

/**
 * 批量发送消息（简化版）
 */
async function sendBurstMessages(
  context: ToolContext,
  params: Record<string, unknown>
): Promise<ToolResult> {
  const membersWithTestUser = getMembersWithTestUser(context)
  const messageCount = params.messageCount as number
  const interval = params.interval as number
  const useRandomMessage = params.useRandomMessage as boolean
  const wsStore = useMultiUserWebSocketStore()

  if (membersWithTestUser.length === 0) {
    return { success: false, message: '房间中没有可操控的测试用户' }
  }

  // 检查在线状态
  const onlineUsers = membersWithTestUser.filter(m => m.testUser && wsStore.isUserConnected(m.testUser.id))
  if (onlineUsers.length === 0) {
    return { success: false, message: '没有用户在线，请先完成认证' }
  }

  // 确保所有在线用户已加入房间
  for (const member of onlineUsers) {
    if (member.testUser && !wsStore.isUserInRoom(member.testUser.id, context.roomId)) {
      wsStore.joinRoom(member.testUser.id, context.roomId)
    }
  }
  await delay(500)

  // 获取已加入房间的用户
  const usersInRoom = onlineUsers.filter(
    m => m.testUser && wsStore.isUserInRoom(m.testUser.id, context.roomId)
  )

  if (usersInRoom.length === 0) {
    return { success: false, message: '没有用户成功加入房间，无法发送消息' }
  }

  const results: { user: string; messages: number; success: number }[] = []

  for (const member of usersInRoom) {
    if (!member.testUser) continue

    let successCount = 0

    for (let i = 0; i < messageCount; i++) {
      const message = useRandomMessage ? getRandomMessage() : `测试消息 ${i + 1}`

      if (wsStore.sendChatMessage(member.testUser.id, context.roomId, message)) {
        successCount++
      }

      if (i < messageCount - 1 && interval > 0) {
        await delay(interval)
      }
    }

    results.push({
      user: member.username,
      messages: messageCount,
      success: successCount,
    })
  }

  const totalSuccess = results.reduce((sum, r) => sum + r.success, 0)
  const totalMessages = usersInRoom.length * messageCount

  return {
    success: totalSuccess > 0,
    message: `批量发送完成: ${totalSuccess}/${totalMessages} 成功`,
    data: { results },
  }
}

/**
 * 消息测试工具执行函数映射
 */
export const messageTestFunctions: Record<string, (context: ToolContext, params: Record<string, unknown>) => Promise<ToolResult>> = {
  single: sendSingleMessage,
  random: sendRandomMessages,
  burst: sendBurstMessages,
}
