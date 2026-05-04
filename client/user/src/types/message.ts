export type MessageType = 'text' | 'image' | 'file'

export interface MessageSender {
  id: string
  username: string
  avatar_url: string | null
}

export interface ReplyToMessage {
  id: string
  sender: MessageSender
  content: string
  created_at: string
}

export interface Message {
  id: string
  room_id: string
  sender: MessageSender
  content: string
  message_type: MessageType
  reply_to: string | null
  reply_to_message: ReplyToMessage | null
  is_deleted: boolean
  created_at: string
  edit_count: number
  edited_at: string | null

  sending?: boolean
  error?: boolean
}
