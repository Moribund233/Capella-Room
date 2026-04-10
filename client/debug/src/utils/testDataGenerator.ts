/**
 * 测试数据生成器
 * 用于生成端到端测试所需的模拟数据
 */

// import type { User, Room, Message } from '@/types/api'

/** 测试用户配置 */
export interface TestUserConfig {
  prefix: string
  count: number
  password: string
}

/** 测试房间配置 */
export interface TestRoomConfig {
  prefix: string
  count: number
  isPrivate: boolean
}

/** 测试消息配置 */
export interface TestMessageConfig {
  count: number
  contentPool: string[]
}

/** 生成的测试数据集 */
export interface TestDataSet {
  users: Array<{
    username: string
    email: string
    password: string
  }>
  rooms: Array<{
    name: string
    description: string
    is_private: boolean
  }>
  messages: string[]
}

// 常用中文词汇池
const chineseWords = [
  '天空', '大地', '海洋', '森林', '山川', '河流', '星辰', '月亮', '太阳', '云彩',
  '花朵', '树木', '草地', '鸟儿', '鱼儿', '蝴蝶', '蜜蜂', '蚂蚁', '蜘蛛', '蜻蜓',
  '春天', '夏天', '秋天', '冬天', '清晨', '黄昏', '夜晚', '黎明', '正午', '午夜',
  '快乐', '幸福', '美好', '温馨', '和谐', '宁静', '安详', '舒适', '惬意', '愉悦',
  '勇敢', '坚强', '智慧', '善良', '真诚', '热情', '勤奋', '努力', '进取', '创新',
]

// 常用句子模板
const sentenceTemplates = [
  '今天的{word}真{adj}！',
  '我喜欢在{word}里{action}。',
  '{word}让人感到{adj}。',
  '大家一起{action}吧！',
  '这个{word}怎么样？',
  '我觉得{word}很{adj}。',
  '有人在{word}吗？',
  '欢迎来到{word}！',
  '{word}是个不错的地方。',
  '祝大家{adj}！',
]

// 动作词池
const actionWords = [
  '聊天', '交流', '分享', '学习', '工作', '休息', '玩耍', '探索', '发现', '创造',
  '思考', '讨论', '合作', '帮助', '支持', '鼓励', '赞美', '感谢', '祝福', '期待',
]

// 形容词池
const adjectiveWords = [
  '美丽', '漂亮', '好看', '精彩', '优秀', '出色', '完美', '很棒', '不错', '挺好',
  '开心', '高兴', '快乐', '愉快', '兴奋', '激动', '感动', '温暖', '舒服', '满意',
]

/**
 * 生成随机整数
 * @param min 最小值
 * @param max 最大值
 * @returns 随机整数
 */
export function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

/**
 * 从数组中随机选择一项
 * @param array 数组
 * @returns 随机项
 */
export function randomPick<T>(array: T[]): T | undefined {
  if (array.length === 0) return undefined
  return array[Math.floor(Math.random() * array.length)]
}

/**
 * 生成随机用户名
 * @param prefix 前缀
 * @param index 索引
 * @returns 用户名
 */
export function generateUsername(prefix: string, index: number): string {
  const word = randomPick(chineseWords)
  return `${prefix}_${word}_${index.toString().padStart(3, '0')}`
}

/**
 * 生成随机邮箱
 * @param username 用户名
 * @returns 邮箱地址
 */
export function generateEmail(username: string): string {
  const domains = ['test.com', 'example.com', 'e2e.local', 'seredeli.test']
  return `${username.toLowerCase()}@${randomPick(domains)}`
}

/**
 * 生成随机房间名
 * @param prefix 前缀
 * @param index 索引
 * @returns 房间名
 */
export function generateRoomName(prefix: string, index: number): string {
  const word = randomPick(chineseWords)
  const suffixes = ['交流室', '讨论组', '聊天室', '活动室', '会议室']
  return `${prefix}_${word}${randomPick(suffixes)}_${index.toString().padStart(2, '0')}`
}

/**
 * 生成房间描述
 * @param roomName 房间名
 * @returns 描述
 */
export function generateRoomDescription(roomName: string): string {
  const templates = [
    `这是一个关于${roomName}的专属空间，欢迎大家加入交流！`,
    `欢迎来到${roomName}，这里是分享和讨论的好地方。`,
    `${roomName} - 让我们一起探索、学习、成长。`,
    `加入${roomName}，与志同道合的朋友一起交流。`,
    `这是${roomName}，一个开放、友好的交流社区。`,
  ]
  return randomPick(templates) ?? '欢迎来到这个房间！'
}

/**
 * 生成随机消息内容
 * @returns 消息内容
 */
export function generateMessageContent(): string {
  const template = randomPick(sentenceTemplates) ?? '{word}让人感到{adj}。'
  const word = randomPick(chineseWords) ?? '生活'
  const adj = randomPick(adjectiveWords) ?? '美好'
  const action = randomPick(actionWords) ?? '交流'
  return template
    .replace('{word}', word)
    .replace('{adj}', adj)
    .replace('{action}', action)
}

/**
 * 生成测试用户列表
 * @param config 用户配置
 * @returns 用户列表
 */
export function generateTestUsers(config: TestUserConfig): Array<{
  username: string
  email: string
  password: string
}> {
  const users = []
  for (let i = 1; i <= config.count; i++) {
    const username = generateUsername(config.prefix, i)
    users.push({
      username,
      email: generateEmail(username),
      password: config.password,
    })
  }
  return users
}

/**
 * 生成测试房间列表
 * @param config 房间配置
 * @returns 房间列表
 */
export function generateTestRooms(config: TestRoomConfig): Array<{
  name: string
  description: string
  is_private: boolean
}> {
  const rooms = []
  for (let i = 1; i <= config.count; i++) {
    const name = generateRoomName(config.prefix, i)
    rooms.push({
      name,
      description: generateRoomDescription(name),
      is_private: config.isPrivate,
    })
  }
  return rooms
}

/**
 * 生成测试消息列表
 * @param config 消息配置
 * @returns 消息内容列表
 */
export function generateTestMessages(config: TestMessageConfig): string[] {
  const messages = []
  for (let i = 0; i < config.count; i++) {
    messages.push(generateMessageContent())
  }
  return messages
}

/**
 * 生成完整测试数据集
 * @param userConfig 用户配置
 * @param roomConfig 房间配置
 * @param messageConfig 消息配置
 * @returns 测试数据集
 */
export function generateTestDataSet(
  userConfig: TestUserConfig = { prefix: 'e2e_user', count: 5, password: 'Test@123456' },
  roomConfig: TestRoomConfig = { prefix: 'e2e', count: 3, isPrivate: false },
  messageConfig: TestMessageConfig = { count: 20, contentPool: [] }
): TestDataSet {
  return {
    users: generateTestUsers(userConfig),
    rooms: generateTestRooms(roomConfig),
    messages: generateTestMessages(messageConfig),
  }
}

/**
 * 生成压力测试数据
 * @param userCount 用户数量
 * @param roomCount 房间数量
 * @param messagesPerRoom 每房间消息数
 * @returns 压力测试数据集
 */
export function generateStressTestData(
  userCount: number = 100,
  roomCount: number = 10,
  messagesPerRoom: number = 1000
): TestDataSet {
  return {
    users: generateTestUsers({
      prefix: 'stress_user',
      count: userCount,
      password: 'Stress@123',
    }),
    rooms: generateTestRooms({
      prefix: 'stress',
      count: roomCount,
      isPrivate: false,
    }),
    messages: generateTestMessages({
      count: messagesPerRoom,
      contentPool: [],
    }),
  }
}

/**
 * 生成边界测试数据
 * @returns 边界测试数据集
 */
export function generateBoundaryTestData(): {
  longUsername: string
  longRoomName: string
  longMessage: string
  specialChars: string
  unicodeText: string
} {
  return {
    longUsername: 'user_' + 'a'.repeat(100),
    longRoomName: 'room_' + '测试'.repeat(50),
    longMessage: '这是一条很长的消息：' + '内容'.repeat(500),
    specialChars: '!@#$%^&*()_+-=[]{}|;:,.<>?',
    unicodeText: '你好世界🌍 Hello こんにちは 안녕하세요',
  }
}

export default {
  generateTestUsers,
  generateTestRooms,
  generateTestMessages,
  generateTestDataSet,
  generateStressTestData,
  generateBoundaryTestData,
  randomInt,
  randomPick,
}
