package com.capella.room.ui.screen.chat

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.AttachFile
import androidx.compose.material.icons.filled.EmojiEmotions
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Send
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.ui.Alignment
import android.widget.Toast
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.navigation.compose.hiltViewModel
import com.capella.room.data.remote.dto.MessageDto
import com.capella.room.data.remote.websocket.WebSocketConnectionState
import com.capella.room.ui.theme.AccentBlue
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
import kotlinx.coroutines.launch

private val avatarColors = listOf(
    listOf(0xFF7C5CFC, 0xFFE85D9A),
    listOf(0xFF3BA55D, 0xFF4A9EFF),
    listOf(0xFFF09042, 0xFFE85D9A),
    listOf(0xFF4A9EFF, 0xFF7C5CFC),
    listOf(0xFFE85D9A, 0xFFF09042),
    listOf(0xFF7C5CFC, 0xFF4A9EFF),
    listOf(0xFF3BA55D, 0xFFE85D9A),
    listOf(0xFF4A9EFF, 0xFFF09042)
)

@Composable
fun ChatScreen(
    onNavigateBack: () -> Unit,
    viewModel: ChatViewModel = hiltViewModel()
) {
    val state = viewModel.uiState
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()

    // 检测是否不在底部（显示回到底部按钮）
    val showScrollToBottom by remember {
        derivedStateOf {
            listState.firstVisibleItemIndex > 2
        }
    }

    // 记录上次消息数量，用于判断是否是新消息
    val lastMessageCount = remember { mutableIntStateOf(0) }

    // Auto-scroll to bottom on new messages (仅当用户已经在底部时)
    val messages = state.messages
    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) {
            // 只有在以下情况才自动滚动到底部：
            // 1. 首次加载消息
            // 2. 用户已经在底部（firstVisibleItemIndex == 0）
            // 3. 不是加载更多消息（消息数量增加超过1条）
            val isFirstLoad = lastMessageCount.intValue == 0
            val isAtBottom = listState.firstVisibleItemIndex <= 1
            val isNewMessage = messages.size - lastMessageCount.intValue <= 1

            if (isFirstLoad || (isAtBottom && isNewMessage)) {
                listState.animateScrollToItem(0)
            }
            lastMessageCount.intValue = messages.size
        }
    }

    Column(modifier = Modifier.fillMaxSize().background(Background)) {
        // Header
        ChatHeader(
            roomName = state.roomInfo?.name ?: "频道",
            memberCount = state.roomInfo?.memberCount ?: 0,
            onBack = onNavigateBack,
            connectionState = state.connectionState
        )

        // Message list
        Box(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
        ) {
            if (state.isLoading) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text("加载中...", color = Muted)
                }
            } else if (state.messages.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text("暂无消息", color = Muted)
                }
            } else {
                LazyColumn(
                    state = listState,
                    modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                    reverseLayout = true,
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    item { Spacer(modifier = Modifier.height(8.dp)) }

                    items(
                        items = messages.reversed(),
                        key = { it.id }
                    ) { message ->
                        val index = messages.reversed().indexOf(message)
                        val prevMsg = if (index > 0) messages.reversed()[index - 1] else null
                        val showDateDivider = index == 0 || viewModel.shouldShowDateDivider(
                            message.createdAt, prevMsg?.createdAt
                        )

                        if (showDateDivider) {
                            DateDivider(
                                label = viewModel.formatDateLabel(message.createdAt)
                            )
                        }

                        MessageItem(
                            message = message,
                            userColor = getUserColor(message.sender.username),
                            formattedTime = viewModel.formatMessageTime(message.createdAt),
                            isCurrentUser = message.sender.id == state.currentUserId
                        )
                    }

                    // Load more trigger
                    if (state.hasMoreMessages) {
                        item {
                            LoadMoreTrigger(
                                isLoading = state.isLoadingMore,
                                onLoadMore = viewModel::loadMoreMessages
                            )
                        }
                    }

                    item { Spacer(modifier = Modifier.height(8.dp)) }
                }
            }

            // 回到底部按钮
            androidx.compose.animation.AnimatedVisibility(
                visible = showScrollToBottom,
                enter = fadeIn() + slideInVertically(initialOffsetY = { it / 2 }),
                exit = fadeOut() + slideOutVertically(targetOffsetY = { it / 2 }),
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(end = 16.dp, bottom = 8.dp)
            ) {
                ScrollToBottomButton(
                    onClick = {
                        scope.launch {
                            listState.animateScrollToItem(0)
                        }
                    }
                )
            }
        }

        // Emoji panel
        AnimatedVisibility(
            visible = state.showEmojiPanel,
            enter = slideInVertically(initialOffsetY = { it }, animationSpec = tween(200)),
            exit = slideOutVertically(targetOffsetY = { it }, animationSpec = tween(200))
        ) {
            EmojiPanel(onEmojiClick = viewModel::insertEmoji)
        }

        // Typing indicator
        val typingText = viewModel.getTypingIndicatorText()
        if (typingText != null) {
            Text(
                text = typingText,
                fontSize = 12.sp,
                color = Muted,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 4.dp)
            )
        }

        // Input area
        val context = LocalContext.current
        ChatInputArea(
            text = state.inputText,
            onTextChange = viewModel::updateInputText,
            onSend = {
                Toast.makeText(context, "发送按钮点击: text='${state.inputText}', isConnected=${state.connectionState == WebSocketConnectionState.Authenticated}", Toast.LENGTH_SHORT).show()
                viewModel.sendMessage()
            },
            onEmojiToggle = viewModel::toggleEmojiPanel,
            placeholder = if (state.roomInfo != null) "发消息到 #${state.roomInfo.name}" else "发消息",
            isConnected = state.connectionState == WebSocketConnectionState.Authenticated
        )
    }
}

@Composable
private fun ChatHeader(
    roomName: String,
    memberCount: Int,
    onBack: () -> Unit,
    connectionState: WebSocketConnectionState
) {
    val context = LocalContext.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(Background)
            .padding(top = 40.dp, bottom = 8.dp, start = 4.dp, end = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        IconButton(
            onClick = onBack,
            modifier = Modifier
                .size(40.dp)
                .clip(RoundedCornerShape(12.dp))
        ) {
            Icon(
                imageVector = Icons.Default.ArrowBack,
                contentDescription = "返回",
                tint = Foreground
            )
        }

        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = roomName,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = Foreground,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                // Connection status indicator
                val (statusColor, statusText) = when (connectionState) {
                    is WebSocketConnectionState.Authenticated -> AccentGreen to "已连接"
                    is WebSocketConnectionState.Connecting,
                    is WebSocketConnectionState.Authenticating -> AccentOrange to "连接中..."
                    is WebSocketConnectionState.Error -> AccentPink to "连接失败"
                    else -> Muted to "未连接"
                }
                if (connectionState != WebSocketConnectionState.Authenticated) {
                    Spacer(modifier = Modifier.width(8.dp))
                    Box(
                        modifier = Modifier
                            .size(8.dp)
                            .clip(CircleShape)
                            .background(statusColor)
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    Text(
                        text = statusText,
                        fontSize = 11.sp,
                        color = statusColor
                    )
                }
            }
            Text(
                text = "$memberCount 位成员",
                fontSize = 13.sp,
                color = Muted,
                maxLines = 1
            )
        }

        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .clickable { Toast.makeText(context, "搜索", Toast.LENGTH_SHORT).show() },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.Search,
                    contentDescription = "搜索",
                    tint = Muted,
                    modifier = Modifier.size(20.dp)
                )
            }
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .clickable { Toast.makeText(context, "成员", Toast.LENGTH_SHORT).show() },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.Person,
                    contentDescription = "成员",
                    tint = Muted,
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }

    // Bottom border
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(0.5.dp)
            .background(Border)
    )
}

@Composable
private fun DateDivider(label: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .weight(1f)
                .height(0.5.dp)
                .background(Border)
        )
        Text(
            text = label,
            fontSize = 12.sp,
            color = Muted,
            fontWeight = FontWeight.Medium,
            modifier = Modifier.padding(horizontal = 12.dp)
        )
        Box(
            modifier = Modifier
                .weight(1f)
                .height(0.5.dp)
                .background(Border)
        )
    }
}

@Composable
private fun MessageItem(
    message: MessageDto,
    userColor: Color,
    formattedTime: String,
    isCurrentUser: Boolean
) {
    // 气泡颜色定义
    val myBubbleColor = Color(0xFF5865F2) // 当前用户气泡 - 紫色
    val otherBubbleColor = Color(0xFF404249) // 其他用户气泡 - 深灰色
    val bubbleColor = if (isCurrentUser) myBubbleColor else otherBubbleColor

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        horizontalArrangement = if (isCurrentUser) Arrangement.End else Arrangement.Start
    ) {
        if (!isCurrentUser) {
            // 其他用户：左侧显示头像
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(CircleShape)
                    .background(userColor),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = message.sender.username.take(1).uppercase(),
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                    fontSize = 14.sp
                )
            }
            Spacer(modifier = Modifier.width(8.dp))
        }

        // 消息气泡
        Column(
            horizontalAlignment = if (isCurrentUser) Alignment.End else Alignment.Start,
            modifier = Modifier.weight(1f, fill = false)
        ) {
            if (!isCurrentUser) {
                // 其他用户：显示用户名和时间
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Text(
                        text = message.sender.username,
                        fontWeight = FontWeight.SemiBold,
                        fontSize = 13.sp,
                        color = userColor
                    )
                    Text(
                        text = formattedTime,
                        fontSize = 11.sp,
                        color = Muted
                    )
                }
                Spacer(modifier = Modifier.height(2.dp))
            }

            // 气泡容器
            Box(
                modifier = Modifier
                    .clip(
                        RoundedCornerShape(
                            topStart = if (isCurrentUser) 16.dp else 4.dp,
                            topEnd = if (isCurrentUser) 4.dp else 16.dp,
                            bottomStart = 16.dp,
                            bottomEnd = 16.dp
                        )
                    )
                    .background(bubbleColor)
                    .padding(horizontal = 14.dp, vertical = 10.dp)
            ) {
                Text(
                    text = buildMessageText(message.content),
                    fontSize = 15.sp,
                    lineHeight = 20.sp,
                    color = Color.White
                )
            }

            if (isCurrentUser) {
                // 当前用户：显示时间
                Text(
                    text = formattedTime,
                    fontSize = 11.sp,
                    color = Muted,
                    modifier = Modifier.padding(top = 2.dp, end = 4.dp)
                )
            }
        }

        if (isCurrentUser) {
            Spacer(modifier = Modifier.width(8.dp))
            // 当前用户：右侧显示头像
            Box(
                modifier = Modifier
                    .size(36.dp)
                    .clip(CircleShape)
                    .background(userColor),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = message.sender.username.take(1).uppercase(),
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                    fontSize = 14.sp
                )
            }
        }
    }
}

private fun buildMessageText(content: String): AnnotatedString {
    val mentionRegex = Regex("@[a-zA-Z0-9_一-鿿]{1,20}")
    return buildAnnotatedString {
        var lastIndex = 0
        mentionRegex.findAll(content).forEach { match ->
            // Text before mention
            if (match.range.first > lastIndex) {
                append(content.substring(lastIndex, match.range.first))
            }
            // Mention
            withStyle(SpanStyle(color = AccentPurple, fontWeight = FontWeight.Medium)) {
                append(match.value)
            }
            lastIndex = match.range.last + 1
        }
        // Remaining text
        if (lastIndex < content.length) {
            append(content.substring(lastIndex))
        }
    }
}

@Composable
private fun LoadMoreTrigger(
    isLoading: Boolean,
    onLoadMore: () -> Unit
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = !isLoading) { onLoadMore() }
            .padding(vertical = 16.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = if (isLoading) "加载中..." else "加载更多消息",
            fontSize = 13.sp,
            color = AccentPurple
        )
    }
}

@Composable
private fun ScrollToBottomButton(
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .size(44.dp)
            .clip(CircleShape)
            .background(SurfaceElevated)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = Icons.Default.KeyboardArrowDown,
            contentDescription = "回到底部",
            tint = Foreground,
            modifier = Modifier.size(24.dp)
        )
    }
}

@Composable
private fun EmojiPanel(onEmojiClick: (String) -> Unit) {
    val emojis = listOf("👍", "❤️", "😂", "🎉", "🔥", "👏", "😍", "🤔", "👋", "✨",
        "😊", "🥺", "😅", "🙏", "💪", "👀", "😎", "🥳", "💯", "❓")

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(Surface)
            .padding(vertical = 12.dp)
    ) {
        // Horizontal scrollable emoji list
        androidx.compose.foundation.lazy.LazyRow(
            modifier = Modifier.fillMaxWidth(),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            items(emojis) { emoji ->
                Text(
                    text = emoji,
                    fontSize = 28.sp,
                    modifier = Modifier
                        .clip(RoundedCornerShape(12.dp))
                        .background(SurfaceElevated)
                        .clickable { onEmojiClick(emoji) }
                        .padding(horizontal = 12.dp, vertical = 8.dp)
                )
            }
        }
    }
}

@Composable
private fun ChatInputArea(
    text: String,
    onTextChange: (String) -> Unit,
    onSend: () -> Unit,
    onEmojiToggle: () -> Unit,
    placeholder: String,
    isConnected: Boolean
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(Background)
            .padding(start = 8.dp, end = 8.dp, top = 12.dp, bottom = 24.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Emoji button
        IconButton(
            onClick = onEmojiToggle,
            modifier = Modifier.size(40.dp)
        ) {
            Icon(
                imageVector = Icons.Default.EmojiEmotions,
                contentDescription = "表情",
                tint = Muted,
                modifier = Modifier.size(22.dp)
            )
        }

        // Attachment button
        IconButton(
            onClick = { /* TODO: attach file */ },
            modifier = Modifier.size(40.dp)
        ) {
            Icon(
                imageVector = Icons.Default.AttachFile,
                contentDescription = "附件",
                tint = Muted,
                modifier = Modifier.size(22.dp)
            )
        }

        // Text input with rounded background
        BasicTextField(
            value = text,
            onValueChange = onTextChange,
            modifier = Modifier
                .weight(1f)
                .padding(horizontal = 4.dp)
                .heightIn(min = 44.dp, max = 120.dp)
                .clip(RoundedCornerShape(22.dp))
                .background(Surface)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            singleLine = false,
            maxLines = 5,
            textStyle = MaterialTheme.typography.bodyLarge.copy(
                color = Foreground
            ),
            decorationBox = { innerTextField ->
                Box(
                    modifier = Modifier.fillMaxWidth(),
                    contentAlignment = Alignment.CenterStart
                ) {
                    if (text.isEmpty()) {
                        Text(
                            text = placeholder,
                            style = MaterialTheme.typography.bodyLarge,
                            color = Muted.copy(alpha = 0.6f)
                        )
                    }
                    innerTextField()
                }
            }
        )

        // Send button
        IconButton(
            onClick = onSend,
            enabled = text.isNotBlank() && isConnected,
            modifier = Modifier
                .size(40.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(
                    if (text.isNotBlank() && isConnected) AccentPurple
                    else SurfaceElevated
                )
        ) {
            Icon(
                imageVector = Icons.Default.Send,
                contentDescription = "发送",
                tint = if (text.isNotBlank() && isConnected) Color.White else Muted,
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

private fun getUserColor(username: String): Color {
    val hash = username.hashCode().let { if (it == Int.MIN_VALUE) 0 else kotlin.math.abs(it) }
    val colors = avatarColors[hash % avatarColors.size]
    return Color(colors[0])
}
