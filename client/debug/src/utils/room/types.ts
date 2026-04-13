/**
 * 房间测试工具类型定义
 */

import type { VNodeChild } from 'vue'
import type { TestUser } from '@/utils/authUtils'
import type { RoomMember } from '@/api/room'

/**
 * 工具参数选项
 */
export interface ToolParamOption {
  label: string
  value: string | number | boolean
}

/**
 * 工具参数定义
 */
export interface ToolParam {
  key: string
  label: string
  type: 'number' | 'string' | 'boolean' | 'select'
  defaultValue: string | number | boolean
  options?: ToolParamOption[]
  min?: number
  max?: number
  step?: number
  description?: string
}

/**
 * 工具菜单项（第二级菜单）
 */
export interface ToolMenuItem {
  key: string
  label: string
  icon?: () => VNodeChild
  params: ToolParam[]
  description?: string
}

/**
 * 工具定义（第一级菜单）
 */
export interface ToolDefinition {
  key: string
  label: string
  icon?: () => VNodeChild
  description?: string
  menuItems: ToolMenuItem[]
}

/**
 * 房间成员与测试用户的映射
 */
export interface RoomMemberWithTestUser extends RoomMember {
  /** 对应的测试用户信息 */
  testUser?: TestUser
  /** 是否已连接 WebSocket */
  isConnected?: boolean
}

/**
 * 工具执行上下文
 */
export interface ToolContext {
  roomId: string
  /** 房间成员列表（包含对应的测试用户信息） */
  roomMembers: RoomMemberWithTestUser[]
  /** 当前选中的测试用户 */
  currentTestUser: TestUser | null
  /** 发送消息函数（通过 WebSocket） */
  sendMessage: (userId: string, content: string) => Promise<void>
}

/**
 * 工具执行结果
 */
export interface ToolResult {
  success: boolean
  message: string
  data?: unknown
}

/**
 * 工具函数接口
 */
export interface ToolFunction {
  (context: ToolContext, params: Record<string, unknown>): Promise<ToolResult> | ToolResult
}
