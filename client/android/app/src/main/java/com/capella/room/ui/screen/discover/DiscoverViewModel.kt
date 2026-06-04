package com.capella.room.ui.screen.discover

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.remote.api.RoomApi
import com.capella.room.data.remote.dto.RoomDto
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import javax.inject.Inject

data class DiscoverUiState(
    val searchQuery: String = "",
    val publicRooms: List<RoomDto> = emptyList(),
    val isLoading: Boolean = true,
    val joiningRoomId: String? = null,
    val errorMessage: String? = null
)

@HiltViewModel
class DiscoverViewModel @Inject constructor(
    private val roomApi: RoomApi
) : ViewModel() {

    var uiState by mutableStateOf(DiscoverUiState())
        private set

    init {
        loadPublicRooms()
    }

    fun loadPublicRooms(search: String? = null) {
        uiState = uiState.copy(isLoading = true, errorMessage = null)
        viewModelScope.launch {
            try {
                val response = roomApi.getRooms(search = search, limit = 50)
                if (response.isSuccessful && response.body()?.success == true) {
                    val rooms = response.body()?.data?.filter { !it.isPrivate } ?: emptyList()
                    uiState = uiState.copy(
                        publicRooms = rooms,
                        isLoading = false
                    )
                } else {
                    uiState = uiState.copy(isLoading = false)
                }
            } catch (e: Exception) {
                uiState = uiState.copy(isLoading = false, errorMessage = "加载失败")
            }
        }
    }

    fun updateSearchQuery(query: String) {
        uiState = uiState.copy(searchQuery = query)
        loadPublicRooms(query.ifBlank { null })
    }

    fun joinRoom(roomId: String) {
        uiState = uiState.copy(joiningRoomId = roomId)
        viewModelScope.launch {
            try {
                val response = roomApi.joinRoom(roomId)
                if (response.isSuccessful && response.body()?.success == true) {
                    // Remove from discover list after joining
                    uiState = uiState.copy(
                        publicRooms = uiState.publicRooms.filter { it.id != roomId },
                        joiningRoomId = null
                    )
                } else {
                    uiState = uiState.copy(joiningRoomId = null)
                }
            } catch (e: Exception) {
                uiState = uiState.copy(joiningRoomId = null)
            }
        }
    }
}
