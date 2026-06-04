package com.capella.room.ui.screen.main

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.capella.room.ui.components.BottomNavTab
import com.capella.room.ui.components.CapellaBottomBar
import com.capella.room.ui.screen.channels.ChannelsScreen
import com.capella.room.ui.screen.channels.ChannelsViewModel
import com.capella.room.ui.screen.discover.DiscoverScreen
import com.capella.room.ui.theme.AccentPink
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated

@Composable
fun MainScreen(
    onNavigateToChat: (String) -> Unit,
    onLogout: () -> Unit
) {
    var selectedTabIndex by rememberSaveable { mutableIntStateOf(0) }
    val channelsViewModel: ChannelsViewModel = hiltViewModel()

    Column(modifier = Modifier.fillMaxSize().background(Background)) {
        // Content area
        Box(modifier = Modifier.weight(1f).fillMaxSize()) {
            when (BottomNavTab.entries[selectedTabIndex]) {
                BottomNavTab.Home -> {
                    ChannelsScreen(
                        state = channelsViewModel.uiState,
                        onSearchQueryChange = channelsViewModel::updateSearchQuery,
                        onFilterSelected = channelsViewModel::selectFilter,
                        onChannelClick = { channelId -> onNavigateToChat(channelId) },
                        onDmClick = { dmId -> onNavigateToChat(dmId) },
                        onCreateRoomClick = { /* TODO: show create room dialog */ },
                        onProfileClick = { selectedTabIndex = BottomNavTab.Profile.ordinal }
                    )
                }

                BottomNavTab.Messages -> {
                    PlaceholderTab(title = "消息", subtitle = "聊天消息将在这里显示")
                }

                BottomNavTab.Discover -> {
                    DiscoverScreen(onNavigateToChat = onNavigateToChat)
                }

                BottomNavTab.Profile -> {
                    ProfilePlaceholder(onLogout = onLogout)
                }
            }
        }

        // Bottom navigation
        CapellaBottomBar(
            selectedTab = BottomNavTab.entries[selectedTabIndex],
            onTabSelected = { tab -> selectedTabIndex = tab.ordinal }
        )
    }
}

@Composable
private fun PlaceholderTab(title: String, subtitle: String) {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.headlineMedium,
                color = Foreground,
                textAlign = TextAlign.Center
            )
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodyLarge,
                color = Muted,
                textAlign = TextAlign.Center
            )
        }
    }
}

@Composable
private fun ProfilePlaceholder(onLogout: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Spacer(modifier = Modifier.height(48.dp))

        // Avatar
        Box(
            modifier = Modifier
                .size(80.dp)
                .clip(CircleShape)
                .background(
                    Brush.linearGradient(listOf(AccentPurple, AccentPink))
                ),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "U",
                fontWeight = FontWeight.Bold,
                color = Color.White,
                style = MaterialTheme.typography.headlineLarge
            )
        }

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = "用户名",
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold,
            color = Foreground
        )

        Text(
            text = "user@example.com",
            style = MaterialTheme.typography.bodyMedium,
            color = Muted
        )

        Spacer(modifier = Modifier.height(32.dp))

        // Stats row
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            StatItem("消息", "0")
            StatItem("频道", "0")
            StatItem("好友", "0")
        }

        Spacer(modifier = Modifier.height(32.dp))

        // Settings section
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(Surface)
                .padding(16.dp)
        ) {
            Column {
                Text(
                    text = "设置",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = Foreground
                )
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "偏好、通知、账号管理",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted
                )
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Logout button
        Button(
            onClick = onLogout,
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = AccentPink.copy(alpha = 0.15f)
            ),
            shape = RoundedCornerShape(12.dp)
        ) {
            Text(
                text = "退出登录",
                color = AccentPink,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

@Composable
private fun StatItem(label: String, value: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(
            text = value,
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold,
            color = Foreground
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = Muted
        )
    }
}
