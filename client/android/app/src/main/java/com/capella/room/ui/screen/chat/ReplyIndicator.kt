package com.capella.room.ui.screen.chat

import androidx.compose.foundation.background
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
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Reply
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.capella.room.data.remote.dto.MessageDto
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.SurfaceElevated

/**
 * 回复指示器
 * 显示正在回复的消息预览
 */
@Composable
fun ReplyIndicator(
    message: MessageDto,
    onCancel: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 4.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(SurfaceElevated)
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // 左侧回复图标
        Icon(
            imageVector = Icons.Default.Reply,
            contentDescription = null,
            tint = AccentGreen,
            modifier = Modifier.size(20.dp)
        )

        Spacer(modifier = Modifier.width(8.dp))

        // 中间分隔线
        Box(
            modifier = Modifier
                .width(3.dp)
                .height(36.dp)
                .clip(RoundedCornerShape(2.dp))
                .background(AccentGreen)
        )

        Spacer(modifier = Modifier.width(12.dp))

        // 消息内容预览
        Column(
            modifier = Modifier.weight(1f)
        ) {
            Text(
                text = "回复 ${message.sender.username}",
                fontSize = 12.sp,
                fontWeight = FontWeight.Medium,
                color = AccentGreen
            )

            Spacer(modifier = Modifier.height(2.dp))

            Text(
                text = message.content,
                fontSize = 13.sp,
                color = Foreground,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        // 取消按钮
        IconButton(
            onClick = onCancel,
            modifier = Modifier.size(32.dp)
        ) {
            Icon(
                imageVector = Icons.Default.Close,
                contentDescription = "取消回复",
                tint = Muted,
                modifier = Modifier.size(18.dp)
            )
        }
    }
}

/**
 * 消息中的回复引用展示
 */
@Composable
fun ReplyQuote(
    replyToMessage: MessageDto,
    onClick: () -> Unit = {}
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(Background.copy(alpha = 0.5f))
            .padding(horizontal = 12.dp, vertical = 8.dp)
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            // 左侧装饰线
            Box(
                modifier = Modifier
                    .width(3.dp)
                    .height(32.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(Muted.copy(alpha = 0.5f))
            )

            Spacer(modifier = Modifier.width(8.dp))

            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = replyToMessage.sender.username,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Medium,
                    color = Muted
                )

                Spacer(modifier = Modifier.height(2.dp))

                Text(
                    text = replyToMessage.content,
                    fontSize = 13.sp,
                    color = Foreground.copy(alpha = 0.8f),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

/**
 * 回复引用占位符（临时实现）
 */
@Composable
fun ReplyQuotePlaceholder(
    replyToId: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(Background.copy(alpha = 0.5f))
            .padding(horizontal = 12.dp, vertical = 8.dp)
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            // 左侧装饰线
            Box(
                modifier = Modifier
                    .width(3.dp)
                    .height(32.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(Muted.copy(alpha = 0.5f))
            )

            Spacer(modifier = Modifier.width(8.dp))

            Column(
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = "引用消息",
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Medium,
                    color = Muted
                )

                Spacer(modifier = Modifier.height(2.dp))

                Text(
                    text = "查看引用的消息",
                    fontSize = 13.sp,
                    color = Foreground.copy(alpha = 0.8f),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}
