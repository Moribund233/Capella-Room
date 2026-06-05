package com.capella.room.ui.screen.user

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
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
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
import com.capella.room.data.remote.dto.UserDto
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import kotlinx.coroutines.delay

/**
 * 用户搜索页面
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UserSearchScreen(
    onBack: () -> Unit,
    onUserClick: (UserDto) -> Unit,
    viewModel: UserSearchViewModel
) {
    var searchQuery by remember { mutableStateOf("") }
    var searchResults by remember { mutableStateOf<List<UserDto>>(emptyList()) }
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
        searchResults = viewModel.searchUsers(searchQuery)
        isSearching = false
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "搜索用户",
                        fontSize = 18.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = Foreground
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.Default.ArrowBack,
                            contentDescription = "返回",
                            tint = Foreground
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Background
                )
            )
        },
        containerColor = Background
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            // 搜索输入框
            SearchInputField(
                query = searchQuery,
                onQueryChange = { searchQuery = it },
                onClear = { searchQuery = "" },
                placeholder = "搜索用户名..."
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
                            message = "输入用户名搜索用户"
                        )
                    }

                    searchResults.isEmpty() && hasSearched -> {
                        EmptySearchState(
                            message = "未找到包含 \"$searchQuery\" 的用户"
                        )
                    }

                    else -> {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            items(
                                items = searchResults,
                                key = { it.id }
                            ) { user ->
                                UserSearchResultItem(
                                    user = user,
                                    onClick = { onUserClick(user) }
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
 * 用户搜索结果项
 */
@Composable
private fun UserSearchResultItem(
    user: UserDto,
    onClick: () -> Unit
) {
    val userColor = getUserColor(user.username)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(Surface)
            .clickable(onClick = onClick)
            .padding(16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        // 头像
        Box(
            modifier = Modifier
                .size(48.dp)
                .clip(CircleShape)
                .background(userColor),
            contentAlignment = Alignment.Center
        ) {
            if (user.avatarUrl != null) {
                // TODO: 加载网络图片
                Icon(
                    imageVector = Icons.Default.Person,
                    contentDescription = null,
                    tint = androidx.compose.ui.graphics.Color.White,
                    modifier = Modifier.size(24.dp)
                )
            } else {
                Text(
                    text = user.username.take(1).uppercase(),
                    fontWeight = FontWeight.SemiBold,
                    color = androidx.compose.ui.graphics.Color.White,
                    fontSize = 20.sp
                )
            }
        }

        Spacer(modifier = Modifier.width(16.dp))

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = user.username,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = Foreground
            )

            user.email?.let { email ->
                Spacer(modifier = Modifier.height(2.dp))
                Text(
                    text = email,
                    fontSize = 13.sp,
                    color = Muted
                )
            }

            user.status?.let { status ->
                Spacer(modifier = Modifier.height(4.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    // 在线状态指示器
                    Box(
                        modifier = Modifier
                            .size(8.dp)
                            .clip(CircleShape)
                            .background(
                                when (status) {
                                    "online" -> androidx.compose.ui.graphics.Color(0xFF3BA55D)
                                    "away" -> androidx.compose.ui.graphics.Color(0xFFF09042)
                                    "dnd" -> androidx.compose.ui.graphics.Color(0xFFE85D9A)
                                    else -> Muted
                                }
                            )
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = when (status) {
                            "online" -> "在线"
                            "away" -> "离开"
                            "dnd" -> "请勿打扰"
                            else -> "离线"
                        },
                        fontSize = 12.sp,
                        color = Muted
                    )
                }
            }
        }
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
 * 获取用户颜色
 */
private fun getUserColor(username: String): androidx.compose.ui.graphics.Color {
    val colors = listOf(
        0xFF7C5CFC, 0xFF3BA55D, 0xFFF09042, 0xFF4A9EFF,
        0xFFE85D9A, 0xFF7C5CFC, 0xFF3BA55D, 0xFF4A9EFF
    )
    val hash = username.hashCode().let { if (it == Int.MIN_VALUE) 0 else kotlin.math.abs(it) }
    return androidx.compose.ui.graphics.Color(colors[hash % colors.size])
}
