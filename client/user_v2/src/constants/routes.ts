export const ROUTE_NAMES = {
  LOGIN: 'login',
  REGISTER: 'register',
  CHAT: 'chat',
  CHAT_ROOM: 'chat-room',
  PROFILE: 'profile',
  DISCOVER: 'discover',
  FRIENDS: 'friends',
  INVITE: 'invite',
  NOT_FOUND: 'not-found',
} as const

export const ROUTE_PATHS = {
  LOGIN: '/login',
  REGISTER: '/register',
  CHAT: '/',
  CHAT_ROOM: '/room/:roomId',
  PROFILE: '/profile',
  DISCOVER: '/discover',
  FRIENDS: '/friends',
  INVITE: '/invite/:code',
} as const
