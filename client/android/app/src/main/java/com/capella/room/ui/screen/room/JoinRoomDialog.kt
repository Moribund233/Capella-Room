package com.capella.room.ui.screen.room

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentPink
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.SurfaceElevated
import kotlinx.coroutines.delay

/**
 * 加入房间对话框
 */
@Composable
fun JoinRoomDialog(
    onDismiss: () -> Unit,
    onJoin: (String) -> Unit,
    viewModel: RoomInvitationViewModel
) {
    var inviteCode by remember { mutableStateOf("") }
    var isValidating by remember { mutableStateOf(false) }
    var isJoining by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var isValid by remember { mutableStateOf<Boolean?>(null) }

    // 防抖验证邀请码
    LaunchedEffect(inviteCode) {
        if (inviteCode.length < 6) {
            isValid = null
            errorMessage = null
            return@LaunchedEffect
        }

        isValidating = true
        delay(500) // 防抖 500ms

        val valid = viewModel.validateInvite(inviteCode)
        isValid = valid
        errorMessage = if (!valid) "无效的邀请码" else null
        isValidating = false
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceElevated,
        title = {
            Text(
                text = "加入房间",
                fontSize = 18.sp,
                fontWeight = FontWeight.SemiBold,
                color = Foreground
            )
        },
        text = {
            Column {
                Text(
                    text = "输入邀请码加入房间",
                    fontSize = 14.sp,
                    color = Muted,
                    modifier = Modifier.padding(bottom = 16.dp)
                )

                // 邀请码输入框
                BasicTextField(
                    value = inviteCode,
                    onValueChange = { newValue ->
                        // 只允许大写字母和数字，最多8位
                        val filtered = newValue.uppercase()
                            .filter { it.isLetterOrDigit() }
                            .take(8)
                        inviteCode = filtered
                        errorMessage = null
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(Background)
                        .border(
                            width = 1.dp,
                            color = when {
                                isValid == true -> AccentGreen
                                isValid == false -> AccentPink
                                else -> Border
                            },
                            shape = RoundedCornerShape(8.dp)
                        )
                        .padding(horizontal = 16.dp, vertical = 14.dp),
                    singleLine = true,
                    textStyle = TextStyle(
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Medium,
                        color = Foreground,
                        letterSpacing = 4.sp,
                        textAlign = TextAlign.Center
                    ),
                    keyboardOptions = KeyboardOptions(
                        capitalization = KeyboardCapitalization.Characters
                    ),
                    decorationBox = { innerTextField ->
                        Box(
                            modifier = Modifier.fillMaxWidth(),
                            contentAlignment = Alignment.Center
                        ) {
                            if (inviteCode.isEmpty()) {
                                Text(
                                    text = "输入邀请码",
                                    fontSize = 16.sp,
                                    color = Muted.copy(alpha = 0.6f)
                                )
                            }
                            innerTextField()
                        }
                    }
                )

                Spacer(modifier = Modifier.height(8.dp))

                // 验证状态
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    when {
                        isValidating -> {
                            CircularProgressIndicator(
                                modifier = Modifier.size(16.dp),
                                color = AccentPurple,
                                strokeWidth = 2.dp
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "验证中...",
                                fontSize = 12.sp,
                                color = Muted
                            )
                        }
                        isValid == true -> {
                            Text(
                                text = "✓ 邀请码有效",
                                fontSize = 12.sp,
                                color = AccentGreen
                            )
                        }
                        errorMessage != null -> {
                            Text(
                                text = errorMessage!!,
                                fontSize = 12.sp,
                                color = AccentPink
                            )
                        }
                    }
                }
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    isJoining = true
                    onJoin(inviteCode)
                },
                enabled = isValid == true && !isJoining,
                colors = ButtonDefaults.buttonColors(
                    containerColor = AccentPurple,
                    disabledContainerColor = Muted.copy(alpha = 0.3f)
                )
            ) {
                if (isJoining) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(16.dp),
                        color = androidx.compose.ui.graphics.Color.White,
                        strokeWidth = 2.dp
                    )
                } else {
                    Text("加入")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("取消", color = Muted)
            }
        }
    )
}
