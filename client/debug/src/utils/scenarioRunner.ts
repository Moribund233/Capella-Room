/**
 * 场景编排器和测试流程控制器
 * 用于执行端到端测试场景
 */

import { login, register, logout, createRoom, joinRoom, leaveRoom, deleteRoom } from '@/api'
import { wsClient } from '@/api'

/** 测试步骤类型 */
export type TestStepType =
  | 'register'
  | 'login'
  | 'logout'
  | 'createRoom'
  | 'joinRoom'
  | 'leaveRoom'
  | 'deleteRoom'
  | 'sendMessage'
  | 'connectWebSocket'
  | 'disconnectWebSocket'
  | 'wait'
  | 'custom'

/** 测试步骤 */
export interface TestStep {
  id: string
  type: TestStepType
  name: string
  description?: string
  params?: Record<string, unknown>
  delay?: number
  retries?: number
  timeout?: number
  skipOnError?: boolean
}

/** 测试场景 */
export interface TestScenario {
  id: string
  name: string
  description: string
  steps: TestStep[]
  setup?: () => Promise<void>
  teardown?: () => Promise<void>
}

/** 步骤执行结果 */
export interface StepResult {
  stepId: string
  success: boolean
  duration: number
  error?: string
  data?: unknown
  timestamp: Date
}

/** 场景执行结果 */
export interface ScenarioResult {
  scenarioId: string
  success: boolean
  startTime: Date
  endTime: Date
  duration: number
  stepResults: StepResult[]
  summary: {
    total: number
    passed: number
    failed: number
    skipped: number
  }
}

/** 步骤执行上下文 */
export interface StepContext {
  user?: {
    id: string
    username: string
    email: string
    token: string
  }
  room?: {
    id: string
    name: string
  }
  webSocket?: {
    connected: boolean
    joinedRooms: string[]
  }
  data: Record<string, unknown>
}

/** 步骤处理器 */
type StepHandler = (step: TestStep, context: StepContext) => Promise<unknown>

/** 场景运行器 */
export class ScenarioRunner {
  private stepHandlers: Map<TestStepType, StepHandler> = new Map()
  private context: StepContext = { data: {} }
  private abortController: AbortController | null = null
  private isRunning = false

  constructor() {
    this.registerDefaultHandlers()
  }

  /**
   * 注册默认的步骤处理器
   */
  private registerDefaultHandlers(): void {
    this.stepHandlers.set('register', this.handleRegister.bind(this))
    this.stepHandlers.set('login', this.handleLogin.bind(this))
    this.stepHandlers.set('logout', this.handleLogout.bind(this))
    this.stepHandlers.set('createRoom', this.handleCreateRoom.bind(this))
    this.stepHandlers.set('joinRoom', this.handleJoinRoom.bind(this))
    this.stepHandlers.set('leaveRoom', this.handleLeaveRoom.bind(this))
    this.stepHandlers.set('deleteRoom', this.handleDeleteRoom.bind(this))
    this.stepHandlers.set('sendMessage', this.handleSendMessage.bind(this))
    this.stepHandlers.set('connectWebSocket', this.handleConnectWebSocket.bind(this))
    this.stepHandlers.set('disconnectWebSocket', this.handleDisconnectWebSocket.bind(this))
    this.stepHandlers.set('wait', this.handleWait.bind(this))
  }

  /**
   * 注册自定义步骤处理器
   * @param type 步骤类型
   * @param handler 处理器函数
   */
  registerHandler(type: TestStepType, handler: StepHandler): void {
    this.stepHandlers.set(type, handler)
  }

  /**
   * 执行测试场景
   * @param scenario 测试场景
   * @returns 场景执行结果
   */
  async runScenario(scenario: TestScenario): Promise<ScenarioResult> {
    if (this.isRunning) {
      throw new Error('Another scenario is already running')
    }

    this.isRunning = true
    this.abortController = new AbortController()
    this.context = { data: {} }

    const startTime = new Date()
    const stepResults: StepResult[] = []
    let passed = 0
    let failed = 0
    let skipped = 0

    try {
      // 执行 setup
      if (scenario.setup) {
        await scenario.setup()
      }

      // 执行每个步骤
      for (const step of scenario.steps) {
        if (this.abortController.signal.aborted) {
          skipped++
          continue
        }

        const result = await this.executeStep(step)
        stepResults.push(result)

        if (result.success) {
          passed++
        } else {
          failed++
          if (!step.skipOnError) {
            break
          }
        }
      }
    } finally {
      // 执行 teardown
      if (scenario.teardown) {
        try {
          await scenario.teardown()
        } catch (error) {
          console.error('Teardown error:', error)
        }
      }

      this.isRunning = false
    }

    const endTime = new Date()

    return {
      scenarioId: scenario.id,
      success: failed === 0,
      startTime,
      endTime,
      duration: endTime.getTime() - startTime.getTime(),
      stepResults,
      summary: {
        total: scenario.steps.length,
        passed,
        failed,
        skipped,
      },
    }
  }

  /**
   * 执行单个步骤
   * @param step 测试步骤
   * @returns 步骤执行结果
   */
  private async executeStep(step: TestStep): Promise<StepResult> {
    const startTime = Date.now()
    const handler = this.stepHandlers.get(step.type)

    if (!handler) {
      return {
        stepId: step.id,
        success: false,
        duration: Date.now() - startTime,
        error: `No handler registered for step type: ${step.type}`,
        timestamp: new Date(),
      }
    }

    // 执行延迟
    if (step.delay) {
      await this.sleep(step.delay)
    }

    try {
      const data = await this.runWithTimeout(
        () => handler(step, this.context),
        step.timeout || 30000
      )

      return {
        stepId: step.id,
        success: true,
        duration: Date.now() - startTime,
        data,
        timestamp: new Date(),
      }
    } catch (error) {
      return {
        stepId: step.id,
        success: false,
        duration: Date.now() - startTime,
        error: error instanceof Error ? error.message : String(error),
        timestamp: new Date(),
      }
    }
  }

  /**
   * 带超时的函数执行
   * @param fn 函数
   * @param timeout 超时时间
   * @returns 执行结果
   */
  private async runWithTimeout<T>(fn: () => Promise<T>, timeout: number): Promise<T> {
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(new Error(`Step timeout after ${timeout}ms`))
      }, timeout)

      fn()
        .then((result) => {
          clearTimeout(timer)
          resolve(result)
        })
        .catch((error) => {
          clearTimeout(timer)
          reject(error)
        })
    })
  }

  /**
   * 停止当前场景
   */
  stop(): void {
    if (this.abortController) {
      this.abortController.abort()
    }
    this.isRunning = false
  }

  /**
   * 获取当前上下文
   * @returns 步骤上下文
   */
  getContext(): StepContext {
    return { ...this.context }
  }

  /**
   * 设置上下文数据
   * @param key 键
   * @param value 值
   */
  setContextData(key: string, value: unknown): void {
    this.context.data[key] = value
  }

  /**
   * 延迟函数
   * @param ms 毫秒
   * @returns Promise
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms))
  }

  // ========== 默认步骤处理器 ==========

  private async handleRegister(step: TestStep, context: StepContext): Promise<unknown> {
    const { username, email, password } = step.params || {}
    const result = await register({
      username: String(username),
      email: String(email),
      password: String(password)
    })
    context.user = {
      id: result.user.id,
      username: result.user.username,
      email: result.user.email,
      token: result.access_token,
    }
    return result
  }

  private async handleLogin(step: TestStep, context: StepContext): Promise<unknown> {
    const { email, password } = step.params || {}
    const result = await login({
      email: String(email),
      password: String(password)
    })
    context.user = {
      id: result.user.id,
      username: result.user.username,
      email: result.user.email,
      token: result.access_token,
    }
    return result
  }

  private async handleLogout(): Promise<unknown> {
    await logout()
    return { success: true }
  }

  private async handleCreateRoom(step: TestStep, context: StepContext): Promise<unknown> {
    const { name, description, is_private } = step.params || {}
    const result = await createRoom({
      name: String(name),
      description: description ? String(description) : undefined,
      is_private: Boolean(is_private)
    })
    context.room = {
      id: result.id,
      name: result.name,
    }
    return result
  }

  private async handleJoinRoom(step: TestStep, context: StepContext): Promise<unknown> {
    const { roomId } = step.params || {}
    const targetRoomId = String(roomId || context.room?.id)
    if (!targetRoomId) {
      throw new Error('Room ID not provided')
    }
    return await joinRoom(targetRoomId)
  }

  private async handleLeaveRoom(step: TestStep, context: StepContext): Promise<unknown> {
    const { roomId } = step.params || {}
    const targetRoomId = String(roomId || context.room?.id)
    if (!targetRoomId) {
      throw new Error('Room ID not provided')
    }
    return await leaveRoom(targetRoomId)
  }

  private async handleDeleteRoom(step: TestStep, context: StepContext): Promise<unknown> {
    const { roomId } = step.params || {}
    const targetRoomId = String(roomId || context.room?.id)
    if (!targetRoomId) {
      throw new Error('Room ID not provided')
    }
    return await deleteRoom(targetRoomId)
  }

  private async handleSendMessage(step: TestStep, context: StepContext): Promise<unknown> {
    const { roomId, content } = step.params || {}
    const targetRoomId = String(roomId || context.room?.id)
    if (!targetRoomId) {
      throw new Error('Room ID not provided')
    }
    if (!content) {
      throw new Error('Message content not provided')
    }
    return wsClient.sendChatMessage(targetRoomId, String(content))
  }

  private async handleConnectWebSocket(): Promise<unknown> {
    return new Promise((resolve, reject) => {
      wsClient.setHandlers({
        onConnect: () => resolve({ connected: true }),
        onError: (error: Error) => reject(error),
      })
      wsClient.connect().catch(reject)
    })
  }

  private async handleDisconnectWebSocket(): Promise<unknown> {
    wsClient.disconnect()
    return { disconnected: true }
  }

  private async handleWait(step: TestStep): Promise<unknown> {
    const { duration } = step.params || { duration: 1000 }
    await this.sleep(Number(duration))
    return { waited: duration }
  }
}

/**
 * 预定义的测试场景
 */
export const predefinedScenarios: TestScenario[] = [
  {
    id: 'complete-user-flow',
    name: '完整用户流程',
    description: '模拟完整用户流程：注册 → 登录 → 创建房间 → 加入房间 → 发送消息 → 离开房间 → 删除房间',
    steps: [
      {
        id: 'step-1',
        type: 'register',
        name: '用户注册',
        params: { username: 'testuser', email: 'test@example.com', password: 'Test@123456' },
      },
      {
        id: 'step-2',
        type: 'login',
        name: '用户登录',
        params: { email: 'test@example.com', password: 'Test@123456' },
      },
      {
        id: 'step-3',
        type: 'createRoom',
        name: '创建房间',
        params: { name: '测试房间', description: '端到端测试房间', is_private: false },
      },
      {
        id: 'step-4',
        type: 'connectWebSocket',
        name: '连接 WebSocket',
      },
      {
        id: 'step-5',
        type: 'joinRoom',
        name: '加入房间',
      },
      {
        id: 'step-6',
        type: 'sendMessage',
        name: '发送消息',
        params: { content: 'Hello, this is a test message!' },
      },
      {
        id: 'step-7',
        type: 'wait',
        name: '等待消息同步',
        params: { duration: 1000 },
      },
      {
        id: 'step-8',
        type: 'leaveRoom',
        name: '离开房间',
      },
      {
        id: 'step-9',
        type: 'disconnectWebSocket',
        name: '断开 WebSocket',
      },
      {
        id: 'step-10',
        type: 'deleteRoom',
        name: '删除房间',
      },
      {
        id: 'step-11',
        type: 'logout',
        name: '用户登出',
      },
    ],
  },
  {
    id: 'websocket-reconnect',
    name: 'WebSocket 重连测试',
    description: '测试 WebSocket 断线重连后消息不丢失',
    steps: [
      {
        id: 'ws-1',
        type: 'login',
        name: '用户登录',
        params: { email: 'test@example.com', password: 'Test@123456' },
      },
      {
        id: 'ws-2',
        type: 'connectWebSocket',
        name: '连接 WebSocket',
      },
      {
        id: 'ws-3',
        type: 'joinRoom',
        name: '加入房间',
        params: { roomId: 'lobby' },
      },
      {
        id: 'ws-4',
        type: 'sendMessage',
        name: '发送消息 1',
        params: { roomId: 'lobby', content: 'Message before disconnect' },
      },
      {
        id: 'ws-5',
        type: 'disconnectWebSocket',
        name: '断开连接（模拟网络中断）',
      },
      {
        id: 'ws-6',
        type: 'wait',
        name: '等待重连间隔',
        params: { duration: 5000 },
      },
      {
        id: 'ws-7',
        type: 'connectWebSocket',
        name: '重新连接',
      },
      {
        id: 'ws-8',
        type: 'sendMessage',
        name: '发送消息 2（验证重连成功）',
        params: { roomId: 'lobby', content: 'Message after reconnect' },
      },
    ],
  },
]

export default ScenarioRunner
