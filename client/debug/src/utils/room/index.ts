/**
 * 房间测试工具入口
 * 导出所有工具定义和执行函数
 */

export * from './types'
export { messageTestTool, messageTestFunctions } from './messageTestTool'

import type { ToolDefinition } from './types'
import { messageTestTool } from './messageTestTool'

/**
 * 所有可用的工具定义
 */
export const allTools: ToolDefinition[] = [
  messageTestTool,
]

/**
 * 根据工具键获取工具定义
 */
export function getToolByKey(key: string): ToolDefinition | undefined {
  return allTools.find(tool => tool.key === key)
}

/**
 * 根据工具键和菜单项键获取菜单项
 */
export function getToolMenuItem(toolKey: string, menuItemKey: string) {
  const tool = getToolByKey(toolKey)
  if (!tool) return undefined
  return tool.menuItems.find(item => item.key === menuItemKey)
}
