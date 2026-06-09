package com.capella.room.ui.screen.profile

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Smartphone
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentOrange
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated

/**
 * 隐私与安全页面（基础框架）
 *
 * 当前实现：
 * - 概览：登录设备数 / 异常登录数 / 单设备登录开关
 * - 设备列表（占位：当前设备 + 模拟的 1-2 个其他设备）
 * - 退出全部其他设备（占位）
 * - 登录历史（占位）
 *
 * Phase 8.1 之后将扩展为：
 * - 真实接入 `GET /api/v1/users/me/security/overview` & `/devices`
 * - 解析异常登录
 */
@Composable
fun SecurityScreen(
    onBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    var showTerminateAllDialog by remember { mutableStateOf(false) }
    var isLoading by remember { mutableStateOf(false) }
    var overview by remember { mutableStateOf<SecurityOverview?>(null) }
    var devices by remember { mutableStateOf<List<DeviceInfo>>(emptyList()) }

    LaunchedEffect(Unit) {
        // TODO: 真实接入
        // profileRepository.getSecurityOverview() + getDevices()
        isLoading = true
        try {
            overview = SecurityOverview(
                totalDevices = 1,
                suspiciousLogins = 0,
                singleDeviceLogin = false
            )
            devices = listOf(
                DeviceInfo(
                    id = "current",
                    name = "当前设备",
                    type = "Android",
                    ipAddress = "—",
                    lastActive = "刚刚",
                    isCurrent = true
                )
            )
        } finally {
            isLoading = false
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(Background)
            .verticalScroll(rememberScrollState())
    ) {
        // 顶栏
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 8.dp, end = 16.dp, top = 8.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            IconButton(onClick = onBack) {
                Icon(
                    imageVector = Icons.Default.ArrowBack,
                    contentDescription = "返回",
                    tint = Foreground
                )
            }
            Text(
                text = "隐私与安全",
                style = MaterialTheme.typography.titleLarge,
                color = Foreground
            )
        }

        if (isLoading) {
            Box(modifier = Modifier.fillMaxSize().padding(64.dp), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = AccentPurple)
            }
            return@Column
        }

        // 概览卡片
        overview?.let { ov ->
            SecurityOverviewCard(overview = ov)
        }

        // 设备列表
        SettingsSection(title = "登录设备") {
            if (devices.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(24.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text("暂无设备", color = Muted, fontSize = 13.sp)
                }
            } else {
                devices.forEachIndexed { index, device ->
                    DeviceRow(
                        device = device,
                        showDivider = index < devices.lastIndex
                    )
                }
            }
        }

        // 退出全部其他设备
        SettingsSection(title = "会话管理") {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp)
            ) {
                Button(
                    onClick = { showTerminateAllDialog = true },
                    colors = ButtonDefaults.buttonColors(
                        containerColor = AccentOrange.copy(alpha = 0.15f),
                        contentColor = AccentOrange
                    ),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("退出全部其他设备", fontWeight = FontWeight.SemiBold)
                }
            }
        }

        // 登录历史（占位）
        SettingsSection(title = "登录历史") {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.History,
                    contentDescription = null,
                    tint = AccentPurple,
                    modifier = Modifier.size(20.dp)
                )
                Spacer(modifier = Modifier.size(12.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "查看完整登录记录",
                        color = Foreground,
                        style = MaterialTheme.typography.bodyLarge
                    )
                    Text(
                        text = "包括 IP、设备、时间与风险等级",
                        color = Muted,
                        style = MaterialTheme.typography.bodySmall
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(24.dp))
    }

    if (showTerminateAllDialog) {
        AlertDialog(
            onDismissRequest = { showTerminateAllDialog = false },
            title = { Text("退出其他设备", color = Foreground) },
            text = {
                Text(
                    "将立即终止其他所有设备的登录状态。如启用「单设备登录」，下次在新设备登录会自动退出此前的会话。",
                    color = Muted,
                    style = MaterialTheme.typography.bodyMedium
                )
            },
            confirmButton = {
                TextButton(onClick = {
                    // TODO: 调用后端 terminate_other_sessions
                    showTerminateAllDialog = false
                }) {
                    Text("确认", color = AccentOrange, fontWeight = FontWeight.SemiBold)
                }
            },
            dismissButton = {
                TextButton(onClick = { showTerminateAllDialog = false }) {
                    Text("取消", color = Muted)
                }
            },
            containerColor = Surface
        )
    }
}

@Composable
private fun SecurityOverviewCard(overview: SecurityOverview) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 20.dp, vertical = 12.dp)
            .clip(RoundedCornerShape(16.dp))
            .background(Surface)
            .padding(16.dp)
    ) {
        Column {
            Text(
                text = "账号安全概览",
                color = Foreground,
                fontWeight = FontWeight.SemiBold,
                style = MaterialTheme.typography.titleMedium
            )
            Spacer(modifier = Modifier.height(12.dp))
            Row(modifier = Modifier.fillMaxWidth()) {
                OverviewItem(
                    label = "活跃设备",
                    value = overview.totalDevices.toString(),
                    modifier = Modifier.weight(1f)
                )
                OverviewItem(
                    label = "异常登录",
                    value = overview.suspiciousLogins.toString(),
                    modifier = Modifier.weight(1f),
                    valueColor = if (overview.suspiciousLogins > 0) AccentOrange else Foreground
                )
                OverviewItem(
                    label = "单设备模式",
                    value = if (overview.singleDeviceLogin) "已开启" else "关闭",
                    modifier = Modifier.weight(1f),
                    valueColor = if (overview.singleDeviceLogin) AccentGreen else Muted
                )
            }
        }
    }
}

@Composable
private fun OverviewItem(
    label: String,
    value: String,
    modifier: Modifier = Modifier,
    valueColor: androidx.compose.ui.graphics.Color = Foreground
) {
    Column(modifier = modifier, horizontalAlignment = Alignment.CenterHorizontally) {
        Text(
            text = value,
            color = valueColor,
            fontWeight = FontWeight.Bold,
            fontSize = 22.sp,
            style = MaterialTheme.typography.titleLarge
        )
        Spacer(modifier = Modifier.height(2.dp))
        Text(
            text = label,
            color = Muted,
            fontSize = 12.sp
        )
    }
}

@Composable
private fun DeviceRow(device: DeviceInfo, showDivider: Boolean) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(SurfaceElevated),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.Smartphone,
                    contentDescription = null,
                    tint = AccentPurple,
                    modifier = Modifier.size(20.dp)
                )
            }
            Spacer(modifier = Modifier.size(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = device.name,
                        color = Foreground,
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.SemiBold
                    )
                    if (device.isCurrent) {
                        Spacer(modifier = Modifier.size(8.dp))
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(6.dp))
                                .background(AccentGreen.copy(alpha = 0.18f))
                                .padding(horizontal = 6.dp, vertical = 2.dp)
                        ) {
                            Text(
                                text = "本机",
                                color = AccentGreen,
                                fontSize = 10.sp,
                                fontWeight = FontWeight.SemiBold
                            )
                        }
                    }
                }
                Text(
                    text = "${device.type} · ${device.ipAddress}",
                    color = Muted,
                    style = MaterialTheme.typography.bodySmall
                )
                Text(
                    text = "最近活跃：${device.lastActive}",
                    color = Muted,
                    style = MaterialTheme.typography.bodySmall
                )
            }
            if (!device.isCurrent) {
                TextButton(onClick = { /* TODO: terminate this device */ }) {
                    Text("退出", color = AccentOrange)
                }
            }
        }
        if (showDivider) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 68.dp)
                    .height(1.dp)
                    .background(Border)
            )
        }
    }
}

private data class SecurityOverview(
    val totalDevices: Int,
    val suspiciousLogins: Int,
    val singleDeviceLogin: Boolean
)

private data class DeviceInfo(
    val id: String,
    val name: String,
    val type: String,
    val ipAddress: String,
    val lastActive: String,
    val isCurrent: Boolean
)
