import httpClient from '@/services/http'
import type { ApiResponse } from '@/types/api'
import type { ReactionSummary } from '@/types/message'

export const reactionApi = {
  getMessageReactions(messageId: string): Promise<ApiResponse<ReactionSummary[]>> {
    return httpClient.get(`/messages/${messageId}/reactions`)
  },

  addReaction(messageId: string, emoji: string): Promise<ApiResponse<unknown>> {
    return httpClient.post(`/messages/${messageId}/reactions`, { emoji })
  },

  removeReaction(messageId: string, emoji: string): Promise<ApiResponse<unknown>> {
    return httpClient.delete(`/messages/${messageId}/reactions`, { params: { emoji } })
  },
}
