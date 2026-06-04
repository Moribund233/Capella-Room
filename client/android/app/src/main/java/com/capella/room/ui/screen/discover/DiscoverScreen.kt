package com.capella.room.ui.screen.discover

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import com.capella.room.data.remote.dto.RoomDto
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated

@Composable
fun DiscoverScreen(
    onNavigateToChat: (String) -> Unit,
    viewModel: DiscoverViewModel = hiltViewModel()
) {
    val state = viewModel.uiState

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
    ) {
        // Header
        Text(
            text = "发现",
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = Foreground,
            modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 48.dp, bottom = 16.dp)
        )

        // Search bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp)
                .clip(RoundedCornerShape(9999.dp))
                .background(Surface)
                .padding(horizontal = 16.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Default.Search,
                contentDescription = "搜索",
                tint = Muted,
                modifier = Modifier.size(18.dp)
            )
            Spacer(modifier = Modifier.width(10.dp))
            BasicTextField(
                value = state.searchQuery,
                onValueChange = viewModel::updateSearchQuery,
                singleLine = true,
                textStyle = MaterialTheme.typography.bodyMedium.copy(color = Foreground),
                modifier = Modifier
                    .weight(1f)
                    .padding(vertical = 10.dp),
                decorationBox = { innerTextField ->
                    Box {
                        if (state.searchQuery.isEmpty()) {
                            Text(
                                text = "搜索公开频道",
                                style = MaterialTheme.typography.bodyMedium,
                                color = Muted.copy(alpha = 0.6f)
                            )
                        }
                        innerTextField()
                    }
                }
            )
        }

        Spacer(modifier = Modifier.height(8.dp))

        // Room list
        if (state.isLoading) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                Text("加载中...", color = Muted)
            }
        } else if (state.publicRooms.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        text = if (state.searchQuery.isNotBlank()) "未找到匹配的频道"
                        else "暂无可加入的公开频道",
                        color = Muted,
                        style = MaterialTheme.typography.bodyLarge
                    )
                }
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = androidx.compose.foundation.layout.PaddingValues(
                    horizontal = 16.dp, vertical = 8.dp
                ),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                items(state.publicRooms, key = { it.id }) { room ->
                    PublicRoomCard(
                        room = room,
                        isJoining = state.joiningRoomId == room.id,
                        onJoin = { viewModel.joinRoom(room.id) },
                        onOpen = { onNavigateToChat(room.id) }
                    )
                }
            }
        }
    }
}

@Composable
private fun PublicRoomCard(
    room: RoomDto,
    isJoining: Boolean,
    onJoin: () -> Unit,
    onOpen: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Surface)
            .clickable { onOpen() }
            .padding(14.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Room icon
        val iconText = when {
            room.name.length <= 2 -> room.name
            else -> room.name.take(1)
        }
        Box(
            modifier = Modifier
                .size(48.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(
                    Brush.linearGradient(
                        colors = listOf(Color(0xFF7C5CFC), Color(0xFF4A9EFF))
                    )
                ),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = iconText,
                fontWeight = FontWeight.SemiBold,
                color = Color.White,
                fontSize = 20.sp
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        // Room info
        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = room.name,
                    fontWeight = FontWeight.SemiBold,
                    color = Foreground,
                    fontSize = 15.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Spacer(modifier = Modifier.width(6.dp))
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(4.dp))
                        .background(AccentGreen.copy(alpha = 0.15f))
                        .padding(horizontal = 6.dp, vertical = 2.dp)
                ) {
                    Text(
                        text = "${room.memberCount}人",
                        fontSize = 11.sp,
                        color = AccentGreen
                    )
                }
            }
            if (!room.description.isNullOrBlank()) {
                Spacer(modifier = Modifier.height(2.dp))
                Text(
                    text = room.description,
                    fontSize = 13.sp,
                    color = Muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            Spacer(modifier = Modifier.height(2.dp))
            Text(
                text = "创建者: ${room.owner?.username ?: "未知"}",
                fontSize = 12.sp,
                color = Muted
            )
        }

        Spacer(modifier = Modifier.width(8.dp))

        // Join button
        Button(
            onClick = onJoin,
            enabled = !isJoining,
            colors = ButtonDefaults.buttonColors(
                containerColor = AccentPurple
            ),
            shape = RoundedCornerShape(10.dp),
            modifier = Modifier.height(36.dp)
        ) {
            Text(
                text = if (isJoining) "加入中..." else "加入",
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium
            )
        }
    }
}
