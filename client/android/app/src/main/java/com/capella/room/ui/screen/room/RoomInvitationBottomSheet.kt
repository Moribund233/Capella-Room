package com.capella.room.ui.screen.room

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import com.capella.room.ui.theme.Border
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.capella.room.data.remote.dto.InvitationDto
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentPink
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

/**
 * 房间邀请管理底部弹窗
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RoomInvitationBottomSheet(
    roomId: String,
    roomName: String,
    onDismiss: () -> Unit,
    viewModel: RoomInvitationViewModel
) {
    val sheetState = rememberModalBottomSheetState(
        skipPartiallyExpanded = true
    )
    val context = LocalContext.current

    var invitations by remember { mutableStateOf<List<InvitationDto>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }
    var showCreateDialog by remember { mutableStateOf(false) }

    // 加载邀请列表
    LaunchedEffect(roomId) {
        isLoading = true
        invitations = viewModel.getInvitations(roomId)
        isLoading = false
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = Background,
        dragHandle = null
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(top = 16.dp)
        ) {
            // 顶部标题栏
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(
                    onClick = onDismiss,
                    modifier = Modifier.size(40.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.ArrowBack,
                        contentDescription = "返回",
                        tint = Foreground
                    )
                }

                Spacer(modifier = Modifier.width(8.dp))

                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "邀请管理",
                        fontSize = 18.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = Foreground
                    )
                    Text(
                        text = "#$roomName",
                        fontSize = 13.sp,
                        color = Muted
                    )
                }

                // 创建邀请按钮
                IconButton(
                    onClick = { showCreateDialog = true },
                    modifier = Modifier.size(40.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Add,
                        contentDescription = "创建邀请",
                        tint = AccentGreen
                    )
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // 邀请列表
            Box(modifier = Modifier.fillMaxSize()) {
                when {
                    isLoading -> {
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center
                        ) {
                            CircularProgressIndicator(
                                color = AccentPurple,
                                modifier = Modifier.size(32.dp)
                            )
                        }
                    }

                    invitations.isEmpty() -> {
                        EmptyInvitationState()
                    }

                    else -> {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            items(
                                items = invitations,
                                key = { it.id }
                            ) { invitation ->
                                InvitationItem(
                                    invitation = invitation,
                                    onCopy = {
                                        viewModel.copyInviteCode(context, invitation.inviteCode)
                                    },
                                    onShare = {
                                        viewModel.shareInvite(context, invitation.inviteCode, roomName)
                                    },
                                    onRevoke = {
                                        viewModel.revokeInvitation(roomId, invitation.id)
                                        // 刷新列表
                                        invitations = invitations.filter { it.id != invitation.id }
                                    }
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    // 创建邀请对话框
    if (showCreateDialog) {
        CreateInvitationDialog(
            onDismiss = { showCreateDialog = false },
            onCreate = { expiresInHours ->
                viewModel.createInvitation(roomId, expiresInHours) { newInvitation ->
                    invitations = invitations + newInvitation
                }
                showCreateDialog = false
            }
        )
    }
}

/**
 * 邀请项
 */
@Composable
private fun InvitationItem(
    invitation: InvitationDto,
    onCopy: () -> Unit,
    onShare: () -> Unit,
    onRevoke: () -> Unit
) {
    val clipboardManager = LocalClipboardManager.current

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(Surface)
            .padding(16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // 邀请码
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = invitation.inviteCode,
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold,
                color = AccentPurple,
                letterSpacing = 2.sp
            )

            Spacer(modifier = Modifier.height(4.dp))

            Text(
                text = "过期时间: ${formatExpiryTime(invitation.expiresAt)}",
                fontSize = 12.sp,
                color = if (isExpired(invitation.expiresAt)) AccentPink else Muted
            )

            Text(
                text = "使用次数: ${invitation.usedCount}/${invitation.maxUses ?: "∞"}",
                fontSize = 12.sp,
                color = Muted
            )
        }

        // 操作按钮
        Row(
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            IconButton(
                onClick = {
                    clipboardManager.setText(AnnotatedString(invitation.inviteCode))
                    onCopy()
                },
                modifier = Modifier.size(36.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.ContentCopy,
                    contentDescription = "复制",
                    tint = Muted,
                    modifier = Modifier.size(20.dp)
                )
            }

            IconButton(
                onClick = onShare,
                modifier = Modifier.size(36.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Share,
                    contentDescription = "分享",
                    tint = Muted,
                    modifier = Modifier.size(20.dp)
                )
            }

            IconButton(
                onClick = onRevoke,
                modifier = Modifier.size(36.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Delete,
                    contentDescription = "撤销",
                    tint = AccentPink,
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }
}

/**
 * 空邀请状态
 */
@Composable
private fun EmptyInvitationState() {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Box(
                modifier = Modifier
                    .size(64.dp)
                    .clip(CircleShape)
                    .background(Surface),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.Add,
                    contentDescription = null,
                    tint = Muted.copy(alpha = 0.5f),
                    modifier = Modifier.size(32.dp)
                )
            }
            Text(
                text = "暂无邀请链接",
                fontSize = 14.sp,
                color = Muted
            )
            Text(
                text = "点击右上角 + 创建邀请",
                fontSize = 12.sp,
                color = Muted.copy(alpha = 0.7f)
            )
        }
    }
}

/**
 * 创建邀请对话框
 */
@Composable
private fun CreateInvitationDialog(
    onDismiss: () -> Unit,
    onCreate: (Int) -> Unit
) {
    var selectedHours by remember { mutableStateOf(24) }
    val options = listOf(1, 24, 72, 168) // 1小时, 1天, 3天, 7天

    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceElevated,
        title = { Text("创建邀请链接", color = Foreground) },
        text = {
            Column {
                Text(
                    text = "选择过期时间:",
                    fontSize = 14.sp,
                    color = Muted,
                    modifier = Modifier.padding(bottom = 12.dp)
                )

                options.forEach { hours ->
                    val label = when (hours) {
                        1 -> "1 小时"
                        24 -> "1 天"
                        72 -> "3 天"
                        168 -> "7 天"
                        else -> "$hours 小时"
                    }

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { selectedHours = hours }
                            .padding(vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(
                            modifier = Modifier
                                .size(20.dp)
                                .clip(CircleShape)
                                .background(
                                    if (selectedHours == hours) AccentPurple else Surface
                                )
                                .then(
                                    if (selectedHours != hours) {
                                        Modifier.border(2.dp, Border, CircleShape)
                                    } else Modifier
                                ),
                            contentAlignment = Alignment.Center
                        ) {
                            if (selectedHours == hours) {
                                Box(
                                    modifier = Modifier
                                        .size(8.dp)
                                        .clip(CircleShape)
                                        .background(androidx.compose.ui.graphics.Color.White)
                                )
                            }
                        }

                        Spacer(modifier = Modifier.width(12.dp))

                        Text(
                            text = label,
                            fontSize = 14.sp,
                            color = Foreground
                        )
                    }
                }
            }
        },
        confirmButton = {
            Button(
                onClick = { onCreate(selectedHours) },
                colors = ButtonDefaults.buttonColors(
                    containerColor = AccentPurple
                )
            ) {
                Text("创建")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("取消", color = Muted)
            }
        }
    )
}

/**
 * 格式化过期时间
 */
private fun formatExpiryTime(expiresAt: String?): String {
    if (expiresAt == null) return "永不过期"
    return try {
        val instant = Instant.parse(expiresAt)
        val localDateTime = instant.atZone(ZoneId.systemDefault()).toLocalDateTime()
        val formatter = DateTimeFormatter.ofPattern("MM/dd HH:mm")
        localDateTime.format(formatter)
    } catch (e: Exception) {
        expiresAt.take(16).replace("T", " ")
    }
}

/**
 * 检查是否已过期
 */
private fun isExpired(expiresAt: String?): Boolean {
    if (expiresAt == null) return false
    return try {
        val instant = Instant.parse(expiresAt)
        instant.isBefore(Instant.now())
    } catch (e: Exception) {
        false
    }
}


