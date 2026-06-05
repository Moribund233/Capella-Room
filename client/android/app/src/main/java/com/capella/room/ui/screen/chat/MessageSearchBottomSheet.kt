package com.capella.room.ui.screen.chat

import androidx.compose.animation.AnimatedVisibility
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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.capella.room.data.remote.dto.MessageDto
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated
import kotlinx.coroutines.delay

/**
 * 消息搜索底部弹窗
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MessageSearchBottomSheet(
    roomId: String,
    roomName: String,
    onDismiss: () -> Unit,
    onMessageClick: (MessageDto) -> Unit,
    viewModel: ChatViewModel
) {
    val sheetState = rememberModalBottomSheetState(
        skipPartiallyExpanded = true
    )

    var searchQuery by remember { mutableStateOf("") }
    var searchResults by remember { mutableStateOf<List<MessageDto>>(emptyList()) }
    var isSearching by remember { mutableStateOf(false) }
    var hasSearched by remember { mutableStateOf(false) }

    // 防抖搜索
    LaunchedEffect(searchQuery) {
        if (searchQuery.isBlank()) {
            searchResults = emptyList()
            hasSearched = false
            return@LaunchedEffect
        }

        delay(300) // 防抖 300ms

        isSearching = true
        hasSearched = true

        // 先搜索本地缓存
        val localResults = viewModel.searchLocalMessages(roomId, searchQuery)

        // 如果本地结果不足，再搜索远程
        if (localResults.size < 10) {
            val remoteResults = viewModel.searchRemoteMessages(roomId, searchQuery)
            // 合并结果，去重
            val merged = (localResults + remoteResults).distinctBy { it.id }
            searchResults = merged
        } else {
            searchResults = localResults
        }

        isSearching = false
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
                        text = "搜索消息",
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
            }

            // 搜索输入框
            SearchInputField(
                query = searchQuery,
                onQueryChange = { searchQuery = it },
                onClear = { searchQuery = "" },
                placeholder = "搜索消息内容..."
            )

            Spacer(modifier = Modifier.height(16.dp))

            // 搜索结果
            Box(modifier = Modifier.fillMaxSize()) {
                when {
                    isSearching -> {
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

                    searchQuery.isBlank() -> {
                        EmptySearchState(
                            message = "输入关键词搜索消息"
                        )
                    }

                    searchResults.isEmpty() && hasSearched -> {
                        EmptySearchState(
                            message = "未找到包含 \"$searchQuery\" 的消息"
                        )
                    }

                    else -> {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            items(
                                items = searchResults,
                                key = { it.id }
                            ) { message ->
                                SearchResultItem(
                                    message = message,
                                    searchQuery = searchQuery,
                                    onClick = {
                                        onDismiss()
                                        onMessageClick(message)
                                    }
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

/**
 * 搜索输入框
 */
@Composable
private fun SearchInputField(
    query: String,
    onQueryChange: (String) -> Unit,
    onClear: () -> Unit,
    placeholder: String
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp)
            .clip(RoundedCornerShape(24.dp))
            .background(Surface)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = Icons.Default.Search,
            contentDescription = null,
            tint = Muted,
            modifier = Modifier.size(20.dp)
        )

        Spacer(modifier = Modifier.width(12.dp))

        BasicTextField(
            value = query,
            onValueChange = onQueryChange,
            modifier = Modifier.weight(1f),
            singleLine = true,
            textStyle = MaterialTheme.typography.bodyLarge.copy(
                color = Foreground
            ),
            decorationBox = { innerTextField ->
                Box {
                    if (query.isEmpty()) {
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

        AnimatedVisibility(
            visible = query.isNotEmpty(),
            enter = fadeIn(),
            exit = fadeOut()
        ) {
            IconButton(
                onClick = onClear,
                modifier = Modifier.size(24.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Clear,
                    contentDescription = "清除",
                    tint = Muted,
                    modifier = Modifier.size(18.dp)
                )
            }
        }
    }
}

/**
 * 搜索结果项
 */
@Composable
private fun SearchResultItem(
    message: MessageDto,
    searchQuery: String,
    onClick: () -> Unit
) {
    val userColor = getSearchUserColor(message.sender.username)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.Top
    ) {
        // 头像
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape)
                .background(userColor),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = message.sender.username.take(1).uppercase(),
                fontWeight = FontWeight.SemiBold,
                color = androidx.compose.ui.graphics.Color.White,
                fontSize = 16.sp
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            // 用户名和时间
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text(
                    text = message.sender.username,
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 14.sp,
                    color = userColor
                )
                Text(
                    text = formatSearchTime(message.createdAt),
                    fontSize = 12.sp,
                    color = Muted
                )
            }

            Spacer(modifier = Modifier.height(4.dp))

            // 消息内容（高亮搜索词）
            HighlightedText(
                text = message.content,
                highlight = searchQuery,
                maxLines = 2
            )
        }
    }
}

/**
 * 高亮文本
 */
@Composable
private fun HighlightedText(
    text: String,
    highlight: String,
    maxLines: Int
) {
    val lowerText = text.lowercase()
    val lowerHighlight = highlight.lowercase()
    val index = lowerText.indexOf(lowerHighlight)

    if (index == -1) {
        Text(
            text = text,
            fontSize = 14.sp,
            color = Foreground,
            maxLines = maxLines,
            overflow = TextOverflow.Ellipsis
        )
    } else {
        val before = text.substring(0, index)
        val match = text.substring(index, index + highlight.length)
        val after = text.substring(index + highlight.length)

        Text(
            text = buildAnnotatedString {
                append(before)
                withStyle(
                    SpanStyle(
                        background = AccentPurple.copy(alpha = 0.3f),
                        color = Foreground,
                        fontWeight = FontWeight.Medium
                    )
                ) {
                    append(match)
                }
                append(after)
            },
            fontSize = 14.sp,
            color = Foreground,
            maxLines = maxLines,
            overflow = TextOverflow.Ellipsis
        )
    }
}

/**
 * 空搜索状态
 */
@Composable
private fun EmptySearchState(message: String) {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(
                imageVector = Icons.Default.Search,
                contentDescription = null,
                tint = Muted.copy(alpha = 0.5f),
                modifier = Modifier.size(48.dp)
            )
            Text(
                text = message,
                fontSize = 14.sp,
                color = Muted
            )
        }
    }
}

/**
 * 格式化搜索时间
 */
private fun formatSearchTime(createdAt: String): String {
    return try {
        val instant = java.time.Instant.parse(createdAt)
        val localDateTime = java.time.LocalDateTime.ofInstant(
            instant,
            java.time.ZoneId.systemDefault()
        )
        val formatter = java.time.format.DateTimeFormatter.ofPattern("MM/dd HH:mm")
        localDateTime.format(formatter)
    } catch (e: Exception) {
        createdAt.take(16).replace("T", " ")
    }
}

/**
 * 获取用户颜色
 */
private fun getSearchUserColor(username: String): androidx.compose.ui.graphics.Color {
    val colors = listOf(
        0xFF7C5CFC, 0xFF3BA55D, 0xFFF09042, 0xFF4A9EFF,
        0xFFE85D9A, 0xFF7C5CFC, 0xFF3BA55D, 0xFF4A9EFF
    )
    val hash = username.hashCode().let { if (it == Int.MIN_VALUE) 0 else kotlin.math.abs(it) }
    return androidx.compose.ui.graphics.Color(colors[hash % colors.size])
}


