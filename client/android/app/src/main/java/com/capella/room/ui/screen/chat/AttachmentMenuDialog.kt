package com.capella.room.ui.screen.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Image
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.capella.room.ui.theme.AccentBlue
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.SurfaceElevated

/**
 * 附件菜单对话框
 */
@Composable
fun AttachmentMenuDialog(
    onDismiss: () -> Unit,
    onImageClick: () -> Unit,
    onFileClick: () -> Unit
) {
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(
            dismissOnBackPress = true,
            dismissOnClickOutside = true,
            usePlatformDefaultWidth = false
        )
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 32.dp)
                .clip(RoundedCornerShape(16.dp))
                .background(SurfaceElevated)
                .padding(20.dp)
        ) {
            Column {
                Text(
                    text = "选择附件类型",
                    fontSize = 18.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = Foreground,
                    modifier = Modifier.padding(bottom = 16.dp)
                )

                // 图片选项
                AttachmentOption(
                    icon = Icons.Default.Image,
                    iconTint = AccentGreen,
                    title = "图片",
                    description = "发送照片或截图",
                    onClick = onImageClick
                )

                Spacer(modifier = Modifier.height(12.dp))

                // 文件选项
                AttachmentOption(
                    icon = Icons.Default.Description,
                    iconTint = AccentBlue,
                    title = "文件",
                    description = "发送文档或其他文件",
                    onClick = onFileClick
                )

                Spacer(modifier = Modifier.height(16.dp))

                // 取消按钮
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .clickable(onClick = onDismiss)
                        .padding(vertical = 12.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = "取消",
                        fontSize = 14.sp,
                        color = Muted
                    )
                }
            }
        }
    }
}

/**
 * 附件选项项
 */
@Composable
private fun AttachmentOption(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    iconTint: androidx.compose.ui.graphics.Color,
    title: String,
    description: String,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .size(48.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(iconTint.copy(alpha = 0.15f)),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = icon,
                contentDescription = title,
                tint = iconTint,
                modifier = Modifier.size(24.dp)
            )
        }

        Spacer(modifier = Modifier.width(16.dp))

        Column {
            Text(
                text = title,
                fontSize = 16.sp,
                fontWeight = FontWeight.Medium,
                color = Foreground
            )
            Text(
                text = description,
                fontSize = 13.sp,
                color = Muted
            )
        }
    }
}
