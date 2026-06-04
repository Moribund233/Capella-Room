package com.capella.room.ui.screen.channels

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.remote.api.RoomApi
import com.capella.room.data.remote.api.UserApi
import com.capella.room.data.remote.dto.DirectRoomDto
import com.capella.room.data.remote.dto.RoomDto
import com.capella.room.data.repository.LocalRoomRepository
import com.capella.room.data.repository.LocalUserRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import javax.inject.Inject

data class OnlineUserUi(
    val name: String,
    val avatarText: String,
    val gradientStart: Long,
    val gradientEnd: Long,
    val isOnline: Boolean = true
)

data class ChannelItemUi(
    val id: String,
    val name: String,
    val type: ChannelType,
    val iconText: String,
    val gradientStart: Long? = null,
    val gradientEnd: Long? = null,
    val lastMessage: String,
    val lastTime: String,
    val unreadCount: Int = 0,
    val memberCount: Int? = null,
    val status: String? = null
)

enum class ChannelType { CHANNEL, DM }

enum class ChannelFilter(val label: String) {
    ALL("全部"),
    CHANNEL("频道"),
    DM("私信"),
    UNREAD("未读")
}

data class ChannelsUiState(
    val searchQuery: String = "",
    val selectedFilter: ChannelFilter = ChannelFilter.ALL,
    val onlineUsers: List<OnlineUserUi> = emptyList(),
    val channels: List<ChannelItemUi> = emptyList(),
    val dms: List<ChannelItemUi> = emptyList(),
    val isLoading: Boolean = true,
    val errorMessage: String? = null,
    val isOfflineMode: Boolean = false
)

/**
 * 频道列表 ViewModel
 * 支持离线模式，优先从本地数据库加载
 */
@HiltViewModel
class ChannelsViewModel @Inject constructor(
    private val userApi: UserApi,
    private val roomApi: RoomApi,
    private val localRoomRepository: LocalRoomRepository,
    private val localUserRepository: LocalUserRepository
) : ViewModel() {

    var uiState by mutableStateOf(ChannelsUiState())
        private set

    init {
        observeLocalRooms()
        loadData()
    }

    /**
     * 观察本地房间数据变化
     */
    private fun observeLocalRooms() {
        viewModelScope.launch {
            localRoomRepository.getJoinedRoomsFlow()
                .collectLatest { rooms ->
                    val channelItems = rooms.map { room ->
                        ChannelItemUi(
                            id = room.id,
                            name = room.name,
                            type = ChannelType.CHANNEL,
                            iconText = when {
                                room.name.length <= 2 -> room.name
                                else -> room.name.take(1)
                            },
                            gradientStart = channelGradientFor(room.name),
                            gradientEnd = channelGradientEndFor(room.name),
                            lastMessage = room.lastMessageContent?.let { content ->
                                room.lastMessageSenderName?.let { sender ->
                                    "$sender: $content"
                                } ?: content
                            } ?: "暂无消息",
                            lastTime = formatTime(room.lastMessageTime),
                            unreadCount = room.unreadCount,
                            memberCount = room.memberCount
                        )
                    }
                    uiState = uiState.copy(
                        channels = channelItems,
                        isLoading = false
                    )
                }
        }
    }

    /**
     * 加载数据
     * 优先从网络获取，失败时显示本地数据
     */
    fun loadData() {
        uiState = uiState.copy(isLoading = true, errorMessage = null)
        viewModelScope.launch {
            try {
                val roomsResponse = userApi.getMyRooms()
                val directRoomsResponse = roomApi.getDirectRoomList()

                if (roomsResponse.isSuccessful && roomsResponse.body()?.success == true) {
                    val rooms = roomsResponse.body()?.data ?: emptyList()
                    val directRooms = if (directRoomsResponse.isSuccessful)
                        directRoomsResponse.body()?.data ?: emptyList()
                    else emptyList()

                    // 保存到本地数据库
                    localRoomRepository.saveRooms(rooms, isJoined = true)

                    // 转换显示数据
                    val channelItems = rooms
                        .filter { it.name != null && !it.name.isNullOrBlank() }
                        .map { it.toChannelItem() }

                    val dmItems = directRooms.map { it.toDmItem() }

                    uiState = uiState.copy(
                        channels = channelItems,
                        dms = dmItems,
                        isLoading = false,
                        errorMessage = null,
                        isOfflineMode = false
                    )
                } else {
                    // API 返回错误，使用本地数据
                    loadFromLocal()
                }
            } catch (e: Exception) {
                // 网络错误，使用本地数据
                loadFromLocal(e.message)
            }
        }
    }

    /**
     * 从本地数据库加载
     */
    private fun loadFromLocal(errorMsg: String? = null) {
        viewModelScope.launch {
            val localRooms = localRoomRepository.getJoinedRooms()

            if (localRooms.isNotEmpty()) {
                // 有本地数据，显示本地数据
                uiState = uiState.copy(
                    isLoading = false,
                    errorMessage = errorMsg ?: "无法连接到服务器，显示本地缓存数据",
                    isOfflineMode = true
                )
            } else {
                // 没有本地数据，显示空状态
                uiState = uiState.copy(
                    isLoading = false,
                    errorMessage = errorMsg ?: "无法连接到服务器，请检查网络",
                    isOfflineMode = true,
                    channels = emptyList(),
                    dms = emptyList()
                )
            }
        }
    }

    private fun RoomDto.toChannelItem(): ChannelItemUi {
        val iconType = when {
            name.length <= 2 -> name
            else -> name.take(1)
        }
        return ChannelItemUi(
            id = id,
            name = name,
            type = ChannelType.CHANNEL,
            iconText = iconType,
            gradientStart = channelGradientFor(name),
            gradientEnd = channelGradientEndFor(name),
            lastMessage = lastMessage?.let { "${it.senderName}: ${it.content}" }
                ?: "暂无消息",
            lastTime = formatTime(lastMessage?.createdAt),
            unreadCount = unreadCount,
            memberCount = memberCount
        )
    }

    private fun DirectRoomDto.toDmItem(): ChannelItemUi {
        return ChannelItemUi(
            id = id,
            name = targetUser.username,
            type = ChannelType.DM,
            iconText = targetUser.username.take(1).uppercase(),
            gradientStart = 0xFF7C5CFC,
            gradientEnd = 0xFFE85D9A,
            lastMessage = "暂无消息",
            lastTime = "",
            unreadCount = 0,
            status = "online"
        )
    }

    fun updateSearchQuery(query: String) {
        uiState = uiState.copy(searchQuery = query)
    }

    fun selectFilter(filter: ChannelFilter) {
        uiState = uiState.copy(selectedFilter = filter)
    }

    fun refresh() {
        loadData()
    }

    companion object {
        private fun channelGradientFor(name: String): Long? = when {
            name.contains("design", ignoreCase = true) -> 0xFF7C5CFC
            name.contains("dev", ignoreCase = true) -> 0xFF3BA55D
            name.contains("音乐", ignoreCase = true) || name.contains("music", ignoreCase = true) -> 0xFFE85D9A
            name.contains("游戏", ignoreCase = true) || name.contains("gaming", ignoreCase = true) -> 0xFF4A9EFF
            name.contains("random", ignoreCase = true) -> 0xFFF09042
            else -> null
        }

        private fun channelGradientEndFor(name: String): Long? = when {
            name.contains("design", ignoreCase = true) -> 0xFF4A9EFF
            name.contains("dev", ignoreCase = true) -> 0xFF4A9EFF
            name.contains("音乐", ignoreCase = true) || name.contains("music", ignoreCase = true) -> 0xFFF09042
            name.contains("游戏", ignoreCase = true) || name.contains("gaming", ignoreCase = true) -> 0xFF7C5CFC
            name.contains("random", ignoreCase = true) -> 0xFFE85D9A
            else -> null
        }

        private fun formatTime(isoString: String?): String {
            if (isoString == null) return ""
            return try {
                val instant = java.time.Instant.parse(isoString)
                val localDateTime = instant.atZone(java.time.ZoneId.systemDefault()).toLocalDateTime()
                val today = java.time.LocalDate.now()
                val messageDate = localDateTime.toLocalDate()

                when {
                    messageDate == today -> {
                        // 今天，显示时间
                        localDateTime.format(java.time.format.DateTimeFormatter.ofPattern("HH:mm"))
                    }
                    messageDate == today.minusDays(1) -> "昨天"
                    messageDate.isAfter(today.minusDays(7)) -> {
                        // 一周内，显示星期
                        localDateTime.format(java.time.format.DateTimeFormatter.ofPattern("EEE"))
                    }
                    else -> {
                        // 更早，显示日期
                        localDateTime.format(java.time.format.DateTimeFormatter.ofPattern("M/d"))
                    }
                }
            } catch (_: Exception) {
                isoString.take(5)
            }
        }
    }
}
