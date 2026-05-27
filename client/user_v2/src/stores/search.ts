import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { searchApi } from '@/api/search'
import { useStorage } from '@vueuse/core'
import type {
  SearchType,
  SearchParams,
  UserSearchItem,
} from '@/types/search'
import type { Room } from '@/types/room'

const SEARCH_HISTORY_KEY = 'capella_search_history'
const MAX_HISTORY_ITEMS = 10

/**
 * 搜索状态管理 Store
 */
export const useSearchStore = defineStore('search', () => {
  // ========== State ==========
  /** 当前搜索类型 */
  const searchType = ref<SearchType>('room')
  /** 当前搜索关键词 */
  const keyword = ref('')
  /** 房间搜索结果 */
  const roomResults = ref<Room[]>([])
  /** 用户搜索结果 */
  const userResults = ref<UserSearchItem[]>([])
  /** 房间结果总数 */
  const roomTotal = ref(0)
  /** 用户结果总数 */
  const userTotal = ref(0)
  /** 加载状态 */
  const loading = ref(false)
  /** 错误信息 */
  const error = ref<string | null>(null)
  /** 搜索历史（持久化存储） */
  const history = useStorage<string[]>(SEARCH_HISTORY_KEY, [])
  /** 当前分页偏移量 */
  const currentOffset = ref(0)
  /** 每页数量 */
  const pageSize = ref(20)

  // ========== Getters ==========
  /** 是否有搜索结果 */
  const hasResults = computed(() => {
    if (searchType.value === 'room') {
      return roomResults.value.length > 0
    }
    return userResults.value.length > 0
  })

  /** 当前类型的结果总数 */
  const totalCount = computed(() => {
    return searchType.value === 'room' ? roomTotal.value : userTotal.value
  })

  /** 是否还有更多结果 */
  const hasMore = computed(() => {
    return currentOffset.value + pageSize.value < totalCount.value
  })

  /** 当前结果列表 */
  const currentResults = computed(() => {
    return searchType.value === 'room' ? roomResults.value : userResults.value
  })

  // ========== Actions ==========

  /**
   * 设置搜索类型
   * @param type 搜索类型
   */
  function setSearchType(type: SearchType) {
    searchType.value = type
    // 切换类型时清空结果
    clearResults()
  }

  /**
   * 设置搜索关键词
   * @param value 关键词
   */
  function setKeyword(value: string) {
    keyword.value = value
  }

  /**
   * 添加到搜索历史
   * @param query 搜索词
   */
  function addToHistory(query: string) {
    if (!query.trim()) return

    // 移除已存在的相同记录
    const index = history.value.indexOf(query)
    if (index > -1) {
      history.value.splice(index, 1)
    }

    // 添加到开头
    history.value.unshift(query)

    // 限制历史记录数量
    if (history.value.length > MAX_HISTORY_ITEMS) {
      history.value = history.value.slice(0, MAX_HISTORY_ITEMS)
    }
  }

  /**
   * 从历史中移除
   * @param query 搜索词
   */
  function removeFromHistory(query: string) {
    const index = history.value.indexOf(query)
    if (index > -1) {
      history.value.splice(index, 1)
    }
  }

  /**
   * 清空搜索历史
   */
  function clearHistory() {
    history.value = []
  }

  /**
   * 清空搜索结果
   */
  function clearResults() {
    roomResults.value = []
    userResults.value = []
    roomTotal.value = 0
    userTotal.value = 0
    currentOffset.value = 0
    error.value = null
  }

  /**
   * 执行搜索
   * @param params 搜索参数
   * @param append 是否追加结果（加载更多）
   * @returns 是否搜索成功
   */
  async function search(params?: Partial<SearchParams>, append = false): Promise<boolean> {
    const searchKeyword = params?.keyword ?? keyword.value
    if (!searchKeyword.trim()) {
      error.value = '请输入搜索关键词'
      return false
    }

    loading.value = true
    error.value = null

    // 如果不是追加，重置偏移量
    if (!append) {
      currentOffset.value = 0
    }

    const limit = params?.limit ?? pageSize.value
    const offset = append ? currentOffset.value : 0

    try {
      if (searchType.value === 'room') {
        const res = await searchApi.searchRooms({
          keyword: searchKeyword,
          limit,
          offset,
        })

        if (res.success && res.data) {
          if (append) {
            roomResults.value.push(...res.data.rooms)
          } else {
            roomResults.value = res.data.rooms
          }
          roomTotal.value = res.data.total
          currentOffset.value = offset + res.data.rooms.length
        } else {
          error.value = res.message || '搜索失败'
          return false
        }
      } else {
        const res = await searchApi.searchUsers({
          keyword: searchKeyword,
          limit,
          offset,
        })

        if (res.success && res.data) {
          if (append) {
            userResults.value.push(...res.data.users)
          } else {
            userResults.value = res.data.users
          }
          userTotal.value = res.data.total
          currentOffset.value = offset + res.data.users.length
        } else {
          error.value = res.message || '搜索失败'
          return false
        }
      }

      // 添加到搜索历史（仅新搜索时）
      if (!append) {
        addToHistory(searchKeyword)
      }

      return true
    } catch (err) {
      error.value = '搜索出错，请稍后重试'
      console.error('[SearchStore] search error:', err)
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * 加载更多结果
   * @returns 是否加载成功
   */
  async function loadMore(): Promise<boolean> {
    if (!hasMore.value || loading.value) return false
    return search({}, true)
  }

  /**
   * 获取公开房间列表（发现页推荐）
   * @param limit 数量限制
   * @returns 房间列表
   */
  async function fetchPublicRooms(limit = 20): Promise<Room[]> {
    loading.value = true
    error.value = null

    try {
      const res = await searchApi.getPublicRooms({ limit, offset: 0 })

      if (res.success && res.data) {
        roomResults.value = res.data
        return res.data
      } else {
        error.value = res.message || '获取房间列表失败'
        return []
      }
    } catch (err) {
      error.value = '获取房间列表出错'
      console.error('[SearchStore] fetchPublicRooms error:', err)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取最近活跃的公开房间
   * @param limit 数量限制
   * @returns 房间列表
   */
  async function fetchRecentRooms(limit = 20): Promise<Room[]> {
    loading.value = true
    error.value = null

    try {
      const res = await searchApi.getRecentPublicRooms({ limit, offset: 0 })

      if (res.success && res.data) {
        roomResults.value = res.data
        return res.data
      } else {
        error.value = res.message || '获取房间列表失败'
        return []
      }
    } catch (err) {
      error.value = '获取房间列表出错'
      console.error('[SearchStore] fetchRecentRooms error:', err)
      return []
    } finally {
      loading.value = false
    }
  }

  /**
   * 重置 Store 状态
   */
  function $reset() {
    searchType.value = 'room'
    keyword.value = ''
    roomResults.value = []
    userResults.value = []
    roomTotal.value = 0
    userTotal.value = 0
    loading.value = false
    error.value = null
    currentOffset.value = 0
    // 注意：不清空 history，因为它是持久化的
  }

  return {
    // State
    searchType,
    keyword,
    roomResults,
    userResults,
    roomTotal,
    userTotal,
    loading,
    error,
    history,
    currentOffset,
    pageSize,
    // Getters
    hasResults,
    totalCount,
    hasMore,
    currentResults,
    // Actions
    setSearchType,
    setKeyword,
    addToHistory,
    removeFromHistory,
    clearHistory,
    clearResults,
    search,
    loadMore,
    fetchPublicRooms,
    fetchRecentRooms,
    $reset,
  }
})
