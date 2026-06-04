package com.capella.room.ui.screen.splash

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.BorderStroke
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
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Forum
import androidx.compose.material.icons.filled.Group
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Timeline
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.sp
import com.capella.room.R
import com.capella.room.ui.theme.AccentBlue
import com.capella.room.ui.theme.AccentGreen
import com.capella.room.ui.theme.AccentOrange
import com.capella.room.ui.theme.AccentPink
import com.capella.room.ui.theme.AccentPurple
import com.capella.room.ui.theme.AccentPurpleSoft
import com.capella.room.ui.theme.Background
import com.capella.room.ui.theme.Border
import com.capella.room.ui.theme.Foreground
import com.capella.room.ui.theme.Muted
import com.capella.room.ui.theme.Surface
import com.capella.room.ui.theme.SurfaceElevated
import kotlinx.coroutines.delay

private const val SPLASH_DURATION_MS = 2200L

@Composable
fun SplashScreen(
    onNavigateToLogin: () -> Unit,
    onNavigateToMain: () -> Unit,
    viewModel: SplashViewModel = hiltViewModel()
) {
    var showSplash by remember { mutableStateOf(true) }
    var splashDone by remember { mutableStateOf(false) }

    val splashAlpha by animateFloatAsState(
        targetValue = if (showSplash) 1f else 0f,
        animationSpec = tween(durationMillis = 400, easing = LinearEasing),
        label = "splashAlpha"
    )

    LaunchedEffect(Unit) {
        delay(SPLASH_DURATION_MS + 400L) // wait for splash + fade out
        splashDone = true
    }

    // Check login state after splash finishes
    LaunchedEffect(splashDone) {
        if (splashDone) {
            viewModel.checkLoginState { isLoggedIn ->
                if (isLoggedIn) onNavigateToMain() else onNavigateToLogin()
            }
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
    ) {
        AnimatedVisibility(
            visible = showSplash,
            exit = fadeOut(animationSpec = tween(400))
        ) {
            SplashOverlay(alpha = splashAlpha)
        }
    }
}

@Composable
private fun SplashOverlay(alpha: Float) {
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val pulseScale by infiniteTransition.animateFloat(
        initialValue = 1f,
        targetValue = 1.08f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = LinearEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulseScale"
    )

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .alpha(alpha),
        contentAlignment = Alignment.Center
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            // Animated logo using vector drawable
            Box(
                modifier = Modifier
                    .size(100.dp)
                    .scale(pulseScale),
                contentAlignment = Alignment.Center
            ) {
                androidx.compose.foundation.Image(
                    painter = painterResource(R.drawable.ic_logo),
                    contentDescription = "Capella Room Logo",
                    modifier = Modifier.size(100.dp)
                )
            }

            Spacer(modifier = Modifier.height(24.dp))

            Text(
                text = "Capella",
                style = MaterialTheme.typography.displayMedium,
                fontWeight = FontWeight.Bold,
                color = Foreground
            )

            Spacer(modifier = Modifier.height(8.dp))

            Text(
                text = "社交聊天，重新想象",
                style = MaterialTheme.typography.bodyLarge,
                color = Muted
            )
        }
    }
}

@Composable
private fun LandingContent(onNavigateToLogin: () -> Unit) {
    Column(
        modifier = Modifier.fillMaxSize()
    ) {
        Column(
            modifier = Modifier
                .weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 24.dp)
        ) {
            Spacer(modifier = Modifier.height(48.dp))

            HeroSection()

            Spacer(modifier = Modifier.height(32.dp))

            ChatPreviewCard()

            Spacer(modifier = Modifier.height(24.dp))

            FeaturesSection()

            Spacer(modifier = Modifier.height(24.dp))
        }

        ActionSection(onNavigateToLogin = onNavigateToLogin)
    }
}

@Composable
private fun HeroSection() {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Logo vector drawable
        Box(
            modifier = Modifier
                .size(80.dp)
                .clip(MaterialTheme.shapes.medium),
            contentAlignment = Alignment.Center
        ) {
            androidx.compose.foundation.Image(
                painter = painterResource(R.drawable.ic_logo),
                contentDescription = "Capella Room Logo",
                modifier = Modifier.size(80.dp)
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = "Capella",
            style = MaterialTheme.typography.displayMedium,
            fontWeight = FontWeight.Bold,
            color = Foreground,
            textAlign = TextAlign.Center
        )

        Spacer(modifier = Modifier.height(12.dp))

        Text(
            text = "一个自然流动的社交聊天空间\n频道、话题和你关心的人",
            style = MaterialTheme.typography.bodyLarge,
            color = Muted,
            textAlign = TextAlign.Center,
            lineHeight = 26.sp
        )
    }
}

@Composable
private fun ChatPreviewCard() {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.large)
            .background(Surface)
            .padding(20.dp)
    ) {
        // Header
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Avatar
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(MaterialTheme.shapes.extraLarge)
                    .background(
                        Brush.linearGradient(
                            colors = listOf(AccentPurple, AccentPink)
                        )
                    ),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "G",
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White
                )
            }

            Spacer(modifier = Modifier.width(12.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "#general",
                    style = MaterialTheme.typography.titleSmall,
                    color = Foreground
                )
                Text(
                    text = "128 位成员",
                    style = MaterialTheme.typography.labelSmall,
                    color = Muted
                )
            }

            Text(
                text = "热门",
                style = MaterialTheme.typography.labelSmall,
                color = AccentPurple,
                modifier = Modifier
                    .clip(MaterialTheme.shapes.extraLarge)
                    .background(AccentPurpleSoft)
                    .padding(horizontal = 10.dp, vertical = 4.dp)
            )
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Bubbles with staggered slide-in
        ChatBubble(
            text = "大家好！今天有什么新动态？",
            isOutgoing = false,
            delayMs = 100
        )

        Spacer(modifier = Modifier.height(8.dp))

        ChatBubble(
            text = "刚刚发布了新功能，快来体验！",
            isOutgoing = true,
            delayMs = 300
        )

        Spacer(modifier = Modifier.height(8.dp))

        ChatBubble(
            text = "太棒了，马上去看看 👍",
            isOutgoing = false,
            delayMs = 500
        )
    }
}

@Composable
private fun ChatBubble(text: String, isOutgoing: Boolean, delayMs: Int = 0) {
    var visible by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        delay(delayMs.toLong())
        visible = true
    }

    val offsetY by animateFloatAsState(
        targetValue = if (visible) 0f else 10f,
        animationSpec = tween(durationMillis = 400, easing = LinearEasing),
        label = "bubbleOffset"
    )

    val bubbleAlpha by animateFloatAsState(
        targetValue = if (visible) 1f else 0f,
        animationSpec = tween(durationMillis = 400),
        label = "bubbleAlpha"
    )

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .alpha(bubbleAlpha)
            .graphicsLayer { translationY = offsetY },
        contentAlignment = if (isOutgoing) Alignment.CenterEnd else Alignment.CenterStart
    ) {
        Text(
            text = text,
            color = if (isOutgoing) Color.White else Foreground,
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier
                .clip(MaterialTheme.shapes.medium)
                .background(
                    if (isOutgoing) AccentPurple else SurfaceElevated
                )
                .padding(horizontal = 16.dp, vertical = 12.dp)
        )
    }
}

@Composable
private fun FeaturesSection() {
    Column(
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            FeatureCard(
                icon = Icons.Default.Forum,
                title = "频道聊天",
                subtitle = "主题讨论空间",
                modifier = Modifier.weight(1f)
            )
            FeatureCard(
                icon = Icons.Default.Timeline,
                title = "话题线程",
                subtitle = "有序的对话",
                modifier = Modifier.weight(1f)
            )
        }

        Spacer(modifier = Modifier.height(12.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            FeatureCard(
                icon = Icons.Default.Group,
                title = "社区连接",
                subtitle = "找到志同道合",
                modifier = Modifier.weight(1f)
            )
            FeatureCard(
                icon = Icons.Default.Shield,
                title = "隐私安全",
                subtitle = "端到端加密",
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun FeatureCard(
    icon: ImageVector,
    title: String,
    subtitle: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .clip(MaterialTheme.shapes.medium)
            .background(Surface)
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .size(44.dp)
                .clip(MaterialTheme.shapes.small)
                .background(AccentPurpleSoft),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = icon,
                contentDescription = title,
                tint = AccentPurple,
                modifier = Modifier.size(22.dp)
            )
        }

        Spacer(modifier = Modifier.height(12.dp))

        Text(
            text = title,
            style = MaterialTheme.typography.titleSmall,
            color = Foreground
        )

        Spacer(modifier = Modifier.height(4.dp))

        Text(
            text = subtitle,
            style = MaterialTheme.typography.bodySmall,
            color = Muted
        )
    }
}

@Composable
private fun ActionSection(onNavigateToLogin: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(Background)
            .padding(horizontal = 24.dp)
            .padding(top = 8.dp, bottom = 32.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Button(
            onClick = onNavigateToLogin,
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = AccentPurple
            ),
            shape = MaterialTheme.shapes.extraLarge
        ) {
            Text(
                text = "开始使用",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
        }

        OutlinedButton(
            onClick = onNavigateToLogin,
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp),
            colors = ButtonDefaults.outlinedButtonColors(
                contentColor = Foreground
            ),
            border = BorderStroke(1.dp, Border),
            shape = MaterialTheme.shapes.extraLarge
        ) {
            Text(
                text = "已有账号？登录",
                style = MaterialTheme.typography.bodyLarge
            )
        }
    }
}
