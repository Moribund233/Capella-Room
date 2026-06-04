package com.capella.room.ui.screen.channels

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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Email
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.IconButton
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
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentOrange
import com.capella.room.ui.theme.AccentPink
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated

@Composable
fun ChannelsScreen(
    state: ChannelsUiState,
    onSearchQueryChange: (String) -> Unit,
    onFilterSelected: (ChannelFilter) -> Unit,
    onChannelClick: (String) -> Unit,
    onDmClick: (String) -> Unit,
    onCreateRoomClick: () -> Unit,
    onProfileClick: () -> Unit
) {
    Box(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            // Header
            ChannelsHeader(onProfileClick = onProfileClick)

            // Scrollable content
            Column(
                modifier = Modifier
                    .weight(1f)
                    .verticalScroll(rememberScrollState())
            ) {
                // Search bar
                SearchBar(
                    query = state.searchQuery,
                    onQueryChange = onSearchQueryChange
                )

                // Online users
                OnlineUsersRow(users = state.onlineUsers)

                // Filter tabs
                FilterTabs(
                    selected = state.selectedFilter,
                    onSelect = onFilterSelected
                )

                // Channel sections - filter based on selected tab
                val showChannels = state.selectedFilter == ChannelFilter.ALL
                        || state.selectedFilter == ChannelFilter.CHANNEL
                        || state.selectedFilter == ChannelFilter.UNREAD
                val showDms = state.selectedFilter == ChannelFilter.ALL
                        || state.selectedFilter == ChannelFilter.DM
                        || state.selectedFilter == ChannelFilter.UNREAD

                val filteredChannels = when (state.selectedFilter) {
                    ChannelFilter.UNREAD -> state.channels.filter { it.unreadCount > 0 }
                    else -> state.channels
                }
                val filteredDms = when (state.selectedFilter) {
                    ChannelFilter.UNREAD -> state.dms.filter { it.unreadCount > 0 }
                    else -> state.dms
                }

                if (showChannels && filteredChannels.isNotEmpty()) {
                    SectionTitle(
                        title = "频道",
                        actionText = "+ 新建",
                        onAction = onCreateRoomClick
                    )
                    ChannelList(
                        channels = filteredChannels,
                        onChannelClick = onChannelClick
                    )
                }

                if (showDms && filteredDms.isNotEmpty()) {
                    SectionTitle(
                        title = "私信",
                        actionText = "+ 新建",
                        onAction = onCreateRoomClick
                    )
                    DmList(
                        dms = filteredDms,
                        onDmClick = onDmClick
                    )
                }

                Spacer(modifier = Modifier.height(80.dp))
            }

            // Bottom nav is handled by MainScreen container
        }

        // FAB
        FloatingActionButton(
            onClick = onCreateRoomClick,
            modifier = Modifier
                .align(Alignment.BottomEnd)
                .padding(end = 20.dp, bottom = 24.dp),
            containerColor = AccentPurple,
            contentColor = Color.White,
            shape = RoundedCornerShape(16.dp)
        ) {
            Icon(
                imageVector = Icons.Default.Add,
                contentDescription = "创建房间",
                modifier = Modifier.size(24.dp)
            )
        }
    }
}

@Composable
private fun ChannelsHeader(onProfileClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(Background)
            .padding(start = 16.dp, end = 16.dp, top = 48.dp, bottom = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = "Capella",
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = Foreground
        )

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            IconButton(
                onClick = { /* TODO: navigate to inbox */ },
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Surface)
            ) {
                Icon(
                    imageVector = Icons.Default.Email,
                    contentDescription = "收件箱",
                    tint = Foreground,
                    modifier = Modifier.size(20.dp)
                )
            }

            IconButton(
                onClick = onProfileClick,
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(Surface)
            ) {
                Icon(
                    imageVector = Icons.Default.Person,
                    contentDescription = "个人资料",
                    tint = Foreground,
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }
}

@Composable
private fun SearchBar(
    query: String,
    onQueryChange: (String) -> Unit
) {
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
            value = query,
            onValueChange = onQueryChange,
            singleLine = true,
            textStyle = MaterialTheme.typography.bodyMedium.copy(
                color = Foreground
            ),
            modifier = Modifier
                .weight(1f)
                .padding(vertical = 10.dp),
            decorationBox = { innerTextField ->
                Box {
                    if (query.isEmpty()) {
                        Text(
                            text = "搜索频道、消息或用户",
                            style = MaterialTheme.typography.bodyMedium,
                            color = Muted.copy(alpha = 0.6f)
                        )
                    }
                    innerTextField()
                }
            }
        )
    }
}

@Composable
private fun OnlineUsersRow(users: List<OnlineUserUi>) {
    LazyRow(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 12.dp),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        items(users, key = { it.name }) { user ->
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.clickable { /* TODO: view profile */ }
            ) {
                Box(
                    modifier = Modifier
                        .size(56.dp)
                        .clip(CircleShape)
                        .background(
                            Brush.linearGradient(
                                colors = listOf(
                                    Color(user.gradientStart),
                                    Color(user.gradientEnd)
                                )
                            )
                        ),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = user.avatarText,
                        fontWeight = FontWeight.SemiBold,
                        color = Color.White,
                        fontSize = 20.sp
                    )

                    // Online indicator
                    if (user.isOnline) {
                        Box(
                            modifier = Modifier
                                .size(14.dp)
                                .clip(CircleShape)
                                .background(AccentGreen)
                                .align(Alignment.BottomEnd)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(6.dp))

                Text(
                    text = user.name,
                    fontSize = 12.sp,
                    color = Muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

@Composable
private fun FilterTabs(
    selected: ChannelFilter,
    onSelect: (ChannelFilter) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp)
            .padding(bottom = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        ChannelFilter.entries.forEach { filter ->
            val isActive = filter == selected
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(9999.dp))
                    .background(
                        if (isActive) AccentPurple
                        else Color.Transparent
                    )
                    .then(
                        if (!isActive) Modifier.background(
                            Surface,
                            RoundedCornerShape(9999.dp)
                        ) else Modifier
                    )
                    .clickable { onSelect(filter) }
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = filter.label,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium,
                    color = if (isActive) Color.White else Muted
                )
            }
        }
    }
}

@Composable
private fun SectionTitle(
    title: String,
    actionText: String? = null,
    onAction: (() -> Unit)? = null
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp)
            .padding(top = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = title,
            fontSize = 13.sp,
            fontWeight = FontWeight.SemiBold,
            color = Muted,
            letterSpacing = 0.5.sp
        )

        if (actionText != null && onAction != null) {
            Text(
                text = actionText,
                fontSize = 13.sp,
                color = AccentPurple,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.clickable { onAction() }
            )
        }
    }
}

@Composable
private fun ChannelList(
    channels: List<ChannelItemUi>,
    onChannelClick: (String) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        channels.forEach { channel ->
            ChannelItem(
                channel = channel,
                onClick = { onChannelClick(channel.id) }
            )
        }
    }
}

@Composable
private fun ChannelItem(
    channel: ChannelItemUi,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Surface)
            .clickable { onClick() }
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Channel icon
        if (channel.gradientStart != null && channel.gradientEnd != null) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(
                        Brush.linearGradient(
                            colors = listOf(
                                Color(channel.gradientStart),
                                Color(channel.gradientEnd)
                            )
                        )
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = channel.iconText,
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                    fontSize = 20.sp
                )
            }
        } else {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(SurfaceElevated),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = channel.iconText,
                    fontWeight = FontWeight.SemiBold,
                    color = Muted,
                    fontSize = 18.sp
                )
            }
        }

        Spacer(modifier = Modifier.width(12.dp))

        // Channel info
        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = channel.name,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = Foreground
                )
                if (channel.type == ChannelType.CHANNEL && channel.gradientStart != null) {
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "#${channel.name}",
                        fontSize = 12.sp,
                        color = Muted
                    )
                }
            }
            Spacer(modifier = Modifier.height(2.dp))
            Text(
                text = channel.lastMessage,
                fontSize = 13.sp,
                color = Muted,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        // Meta (time + badge)
        Column(
            horizontalAlignment = Alignment.End
        ) {
            Text(
                text = channel.lastTime,
                fontSize = 12.sp,
                color = Muted
            )
            if (channel.unreadCount > 0) {
                Spacer(modifier = Modifier.height(4.dp))
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(9999.dp))
                        .background(AccentPurple)
                        .padding(horizontal = 6.dp, vertical = 2.dp)
                ) {
                    Text(
                        text = if (channel.unreadCount > 99) "99+" else channel.unreadCount.toString(),
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = Color.White
                    )
                }
            }
        }
    }
}

@Composable
private fun DmList(
    dms: List<ChannelItemUi>,
    onDmClick: (String) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        dms.forEach { dm ->
            DmItem(
                dm = dm,
                onClick = { onDmClick(dm.id) }
            )
        }
    }
}

@Composable
private fun DmItem(
    dm: ChannelItemUi,
    onClick: () -> Unit
) {
    val isUnread = dm.unreadCount > 0

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .clickable { onClick() }
            .padding(vertical = 10.dp, horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Avatar with status
        Box(modifier = Modifier.size(48.dp)) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(
                        Brush.linearGradient(
                            colors = listOf(
                                Color(dm.gradientStart ?: 0xFF7C5CFC),
                                Color(dm.gradientEnd ?: 0xFFE85D9A)
                            )
                        )
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = dm.iconText,
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                    fontSize = 18.sp
                )
            }

            // Status indicator
            val statusColor = when (dm.status) {
                "online" -> AccentGreen
                "away" -> AccentOrange
                "dnd" -> AccentPink
                else -> Muted
            }
            Box(
                modifier = Modifier
                    .size(12.dp)
                    .clip(CircleShape)
                    .background(statusColor)
                    .align(Alignment.BottomEnd)
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        // DM info
        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = dm.name,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = Foreground
                )
                if (dm.status == "online") {
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = "●",
                        fontSize = 12.sp,
                        color = AccentGreen
                    )
                }
            }
            Spacer(modifier = Modifier.height(2.dp))
            Text(
                text = dm.lastMessage,
                fontSize = 13.sp,
                color = if (isUnread) Foreground else Muted,
                fontWeight = if (isUnread) FontWeight.Medium else FontWeight.Normal,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        // Meta
        Column(horizontalAlignment = Alignment.End) {
            Text(
                text = dm.lastTime,
                fontSize = 12.sp,
                color = Muted
            )
            if (isUnread) {
                Spacer(modifier = Modifier.height(4.dp))
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(9999.dp))
                        .background(AccentPurple)
                        .padding(horizontal = 6.dp, vertical = 2.dp)
                ) {
                    Text(
                        text = dm.unreadCount.toString(),
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = Color.White
                    )
                }
            }
        }
    }
}
