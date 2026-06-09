package com.capella.room.ui.screen.main

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.capella.room.ui.components.BottomNavTab
import com.capella.room.ui.components.CapellaBottomBar
import com.capella.room.ui.screen.channels.ChannelsScreen
import com.capella.room.ui.screen.channels.ChannelsViewModel
import com.capella.room.ui.screen.discover.DiscoverScreen
import com.capella.room.ui.screen.profile.ProfileScreen
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted

@Composable
fun MainScreen(
    onNavigateToChat: (String) -> Unit,
    onNavigateToSecurity: () -> Unit = {},
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
                    PlaceholderTab(
                        title = "消息",
                        subtitle = "聊天消息将在这里显示（Phase 10 通知中心）"
                    )
                }

                BottomNavTab.Discover -> {
                    DiscoverScreen(onNavigateToChat = onNavigateToChat)
                }

                BottomNavTab.Profile -> {
                    ProfileScreen(
                        onLogout = onLogout,
                        onOpenSecurity = onNavigateToSecurity,
                        onOpenAbout = { /* TODO: navigate to about screen */ }
                    )
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
            verticalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(8.dp)
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
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(horizontal = 32.dp)
            )
        }
    }
}
