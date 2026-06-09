package com.capella.room.ui.screen.profile

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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CameraAlt
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Security
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import coil.compose.AsyncImage
import com.capella.room.data.remote.dto.UserDto
import com.capella.room.data.remote.dto.UserSettingsDto
import com.capella.room.data.remote.dto.UserStatsDto
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentOrange
import com.capella.room.ui.theme.AccentPink
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated
import kotlinx.coroutines.flow.collectLatest

/**
 * 个人资料主页面（参照 prototype/android_app/profile.html）
 *
 * 路由：MainScreen → BottomNavTab.Profile
 *
 * 状态机由 [ProfileViewModel] 管理；本组件只负责渲染。
 */
@Composable
fun ProfileScreen(
    onLogout: () -> Unit,
    onOpenSecurity: () -> Unit = {},
    onOpenAbout: () -> Unit = {},
    modifier: Modifier = Modifier,
    viewModel: ProfileViewModel = hiltViewModel()
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val snackbarHost = remember { mutableStateOf<String?>(null) }

    // 事件收集：toast / error / logout
    LaunchedEffect(Unit) {
        viewModel.events.collectLatest { event ->
            when (event) {
                is ProfileEvent.Logout -> onLogout()
                is ProfileEvent.Toast -> snackbarHost.value = event.message
                is ProfileEvent.Error -> snackbarHost.value = event.message
            }
        }
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(Background)
    ) {
        when (val s = state) {
            is ProfileUiState.Loading -> LoadingState()
            is ProfileUiState.Error -> ErrorState(message = s.message, onRetry = viewModel::load)
            is ProfileUiState.Ready -> ReadyContent(
                profile = s.profile,
                stats = s.stats,
                settings = s.settings,
                onEditProfile = { viewModel.updateProfile(it.first, it.second, it.third) },
                onChangePassword = { cur, new -> viewModel.changePassword(cur, new) {} },
                onToggleNotification = viewModel::toggleNotification,
                onToggleSound = viewModel::toggleSound,
                onToggleReadReceipts = viewModel::toggleReadReceipts,
                onToggleSingleDevice = viewModel::toggleSingleDeviceLogin,
                onToggleShowOnline = viewModel::toggleShowOnlineStatus,
                onToggleEnterToSend = viewModel::toggleEnterToSend,
                onOpenSecurity = onOpenSecurity,
                onOpenAbout = onOpenAbout,
                onLogout = viewModel::logout
            )
        }

        // 简易 toast
        snackbarHost.value?.let { msg ->
            Box(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(24.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(SurfaceElevated)
                    .padding(horizontal = 16.dp, vertical = 10.dp)
            ) {
                Text(text = msg, color = Foreground, style = MaterialTheme.typography.bodyMedium)
            }
            // 2.5s 自动消失
            LaunchedEffect(msg) {
                kotlinx.coroutines.delay(2500)
                snackbarHost.value = null
            }
        }
    }
}

// ── 子组件 ──

@Composable
private fun LoadingState() {
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        CircularProgressIndicator(color = AccentPurple)
    }
}

@Composable
private fun ErrorState(message: String, onRetry: () -> Unit) {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(
                imageVector = Icons.Default.Info,
                contentDescription = null,
                tint = Muted,
                modifier = Modifier.size(48.dp)
            )
            Text(
                text = "加载失败",
                style = MaterialTheme.typography.titleMedium,
                color = Foreground
            )
            Text(
                text = message,
                style = MaterialTheme.typography.bodySmall,
                color = Muted,
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(horizontal = 32.dp)
            )
            Button(
                onClick = onRetry,
                colors = ButtonDefaults.buttonColors(containerColor = AccentPurple)
            ) {
                Text("重试")
            }
        }
    }
}

@Composable
private fun ReadyContent(
    profile: UserDto,
    stats: UserStatsDto,
    settings: UserSettingsDto,
    onEditProfile: (Triple<String, String, String>) -> Unit,
    onChangePassword: (String, String) -> Unit,
    onToggleNotification: () -> Unit,
    onToggleSound: () -> Unit,
    onToggleReadReceipts: () -> Unit,
    onToggleSingleDevice: () -> Unit,
    onToggleShowOnline: () -> Unit,
    onToggleEnterToSend: () -> Unit,
    onOpenSecurity: () -> Unit,
    onOpenAbout: () -> Unit,
    onLogout: () -> Unit
) {
    var showEditProfile by remember { mutableStateOf(false) }
    var showChangePassword by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
    ) {
        ProfileHeader(
            profile = profile,
            onEditClick = { showEditProfile = true },
            onChangePasswordClick = { showChangePassword = true }
        )

        StatsCard(
            stats = stats,
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp)
        )

        // ── 偏好设置 ──
        SettingsSection(title = "偏好设置") {
            SettingsToggleRow(
                title = "消息通知",
                checked = settings.notificationEnabled ?: true,
                onCheckedChange = { onToggleNotification() }
            )
            SettingsToggleRow(
                title = "声音提示",
                checked = settings.soundEnabled ?: true,
                onCheckedChange = { onToggleSound() }
            )
            SettingsToggleRow(
                title = "已读回执",
                checked = settings.readReceiptsEnabled ?: true,
                onCheckedChange = { onToggleReadReceipts() },
                showDivider = false
            )
        }

        // ── 隐私 ──
        SettingsSection(title = "隐私") {
            SettingsToggleRow(
                title = "显示在线状态",
                checked = settings.showOnlineStatus ?: true,
                onCheckedChange = { onToggleShowOnline() }
            )
            SettingsToggleRow(
                title = "单设备登录",
                checked = settings.singleDeviceLogin ?: false,
                onCheckedChange = { onToggleSingleDevice() }
            )
            SettingsToggleRow(
                title = "回车发送消息",
                checked = settings.enterToSend ?: true,
                onCheckedChange = { onToggleEnterToSend() },
                showDivider = false
            )
        }

        // ── 账号 ──
        SettingsSection(title = "账号") {
            SettingsNavigationRow(
                icon = Icons.Default.Security,
                title = "隐私与安全",
                subtitle = "管理密码、登录设备、登录记录",
                onClick = onOpenSecurity
            )
            SettingsNavigationRow(
                icon = Icons.Default.Person,
                title = "已连接账号",
                subtitle = "暂未连接第三方账号",
                onClick = { /* TODO: OAuth settings */ }
            )
            SettingsNavigationRow(
                icon = Icons.Default.Info,
                title = "关于 Capella",
                subtitle = "版本 1.0.0",
                onClick = onOpenAbout,
                showDivider = false
            )
        }

        // ── 危险区域 ──
        DangerZone(
            title = "退出登录",
            description = "这将清除你在此设备上的登录状态",
            actionText = "退出登录",
            onAction = onLogout
        )

        Spacer(modifier = Modifier.height(24.dp))
    }

    if (showEditProfile) {
        EditProfileDialog(
            profile = profile,
            onDismiss = { showEditProfile = false },
            onConfirm = { username, email, bio ->
                onEditProfile(Triple(username, email, bio))
                showEditProfile = false
            }
        )
    }

    if (showChangePassword) {
        ChangePasswordDialog(
            onDismiss = { showChangePassword = false },
            onConfirm = { current, new ->
                onChangePassword(current, new)
                showChangePassword = false
            }
        )
    }
}

@Composable
private fun ProfileHeader(
    profile: UserDto,
    onEditClick: () -> Unit,
    onChangePasswordClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 24.dp, bottom = 16.dp),
        contentAlignment = Alignment.TopCenter
    ) {
        // 头像 + 状态指示器 + 编辑按钮
        Box(
            modifier = Modifier.size(100.dp),
            contentAlignment = Alignment.BottomEnd
        ) {
            Box(
                modifier = Modifier
                    .size(100.dp)
                    .clip(CircleShape)
                    .background(Brush.linearGradient(listOf(AccentPurple, AccentPink))),
                contentAlignment = Alignment.Center
            ) {
                if (!profile.avatarUrl.isNullOrBlank()) {
                    AsyncImage(
                        model = profile.avatarUrl,
                        contentDescription = "头像",
                        modifier = Modifier
                            .size(100.dp)
                            .clip(CircleShape)
                    )
                } else {
                    Text(
                        text = profile.username.take(1).uppercase().ifBlank { "U" },
                        fontSize = 40.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White
                    )
                }
            }

            // 状态指示器
            if (profile.status == "online") {
                Box(
                    modifier = Modifier
                        .align(Alignment.BottomStart)
                        .size(24.dp)
                        .clip(CircleShape)
                        .background(AccentGreen)
                )
            }

            // 编辑头像按钮
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(Surface)
                    .clickable(onClick = onEditClick),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.CameraAlt,
                    contentDescription = "编辑头像",
                    tint = Foreground,
                    modifier = Modifier.size(14.dp)
                )
            }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 20.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = profile.username,
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = Foreground
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = "@${profile.username} · ${profile.email}",
            style = MaterialTheme.typography.bodyMedium,
            color = Muted
        )
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = when (profile.status) {
                "online" -> "在线"
                "away" -> "离开"
                "dnd" -> "请勿打扰"
                else -> "离线"
            },
            style = MaterialTheme.typography.bodySmall,
            color = if (profile.status == "online") AccentGreen else Muted
        )
    }

    Spacer(modifier = Modifier.height(16.dp))

    // 操作按钮
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 20.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Button(
            onClick = onEditClick,
            modifier = Modifier
                .weight(1f)
                .height(44.dp),
            colors = ButtonDefaults.buttonColors(containerColor = AccentPurple),
            shape = RoundedCornerShape(9999.dp)
        ) {
            Icon(
                imageVector = Icons.Default.Edit,
                contentDescription = null,
                modifier = Modifier.size(16.dp)
            )
            Spacer(modifier = Modifier.width(6.dp))
            Text("编辑资料", fontSize = 14.sp)
        }
        Button(
            onClick = onChangePasswordClick,
            modifier = Modifier
                .weight(1f)
                .height(44.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = Surface,
                contentColor = Foreground
            ),
            shape = RoundedCornerShape(9999.dp)
        ) {
            Icon(
                imageVector = Icons.Default.Lock,
                contentDescription = null,
                modifier = Modifier.size(16.dp),
                tint = Foreground
            )
            Spacer(modifier = Modifier.width(6.dp))
            Text("修改密码", fontSize = 14.sp)
        }
    }
}

@Composable
private fun StatsCard(
    stats: UserStatsDto,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(Surface)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            StatItem(
                label = "消息",
                value = stats.totalMessages.toString(),
                modifier = Modifier.weight(1f)
            )
            VerticalDivider()
            StatItem(
                label = "频道",
                value = stats.joinedRooms.toString(),
                modifier = Modifier.weight(1f)
            )
            VerticalDivider()
            StatItem(
                label = "在线(h)",
                value = stats.onlineHours.toString(),
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun StatItem(
    label: String,
    value: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .clickable { /* TODO: drill-down */ }
            .padding(vertical = 16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(
            text = value,
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.Bold,
            color = Foreground
        )
        Spacer(modifier = Modifier.height(2.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = Muted
        )
    }
}

@Composable
private fun VerticalDivider() {
    Box(
        modifier = Modifier
            .width(1.dp)
            .height(48.dp)
            .background(com.capella.room.ui.theme.Border)
    )
}
