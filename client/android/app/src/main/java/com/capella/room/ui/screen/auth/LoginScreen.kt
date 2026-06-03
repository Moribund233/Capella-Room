package com.capella.room.ui.screen.auth

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
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

@Composable
fun LoginScreen(
    onNavigateToChannels: () -> Unit,
    onNavigateBack: () -> Unit,
    viewModel: LoginViewModel = hiltViewModel()
) {
    val state = viewModel.uiState
    val focusManager = LocalFocusManager.current

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
    ) {
        // Scrollable content
        Column(
            modifier = Modifier
                .weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 24.dp)
        ) {
            Spacer(modifier = Modifier.height(12.dp))

            // Back button
            IconButton(
                onClick = onNavigateBack,
                modifier = Modifier
                    .size(40.dp)
                    .clip(MaterialTheme.shapes.small)
                    .background(Surface)
            ) {
                Icon(
                    imageVector = Icons.Default.ArrowBack,
                    contentDescription = "返回",
                    tint = Foreground
                )
            }

            Spacer(modifier = Modifier.height(20.dp))

            // Logo section
            LogoSection(isLogin = state.isLoginTab)

            // Error message
            if (state.errorMessage != null) {
                Text(
                    text = state.errorMessage,
                    style = MaterialTheme.typography.bodyMedium,
                    color = AccentPink,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(MaterialTheme.shapes.small)
                        .background(AccentPink.copy(alpha = 0.1f))
                        .padding(12.dp)
                )
                Spacer(modifier = Modifier.height(12.dp))
            }

            Spacer(modifier = Modifier.height(if (state.errorMessage != null) 20.dp else 32.dp))

            // Tab switcher
            TabSwitcher(
                isLogin = state.isLoginTab,
                onSelectLogin = { viewModel.switchToLogin() },
                onSelectRegister = { viewModel.switchToRegister() }
            )

            Spacer(modifier = Modifier.height(24.dp))

            // Form panels
            AnimatedContent(
                targetState = state.isLoginTab,
                transitionSpec = { fadeIn(tween(250)) togetherWith fadeOut(tween(150)) },
                label = "authTab"
            ) { isLogin: Boolean ->
                if (isLogin) {
                    LoginPanel(
                        email = state.loginEmail,
                        onEmailChange = viewModel::updateLoginEmail,
                        password = state.loginPassword,
                        onPasswordChange = viewModel::updateLoginPassword,
                        passwordVisible = state.loginPasswordVisible,
                        onTogglePasswordVisibility = viewModel::toggleLoginPasswordVisibility,
                        rememberMe = state.rememberMe,
                        onRememberMeChange = viewModel::updateRememberMe,
                        isLoading = state.isLoading,
                        onLogin = {
                            focusManager.clearFocus()
                            viewModel.login(onSuccess = onNavigateToChannels)
                        }
                    )
                } else {
                    RegisterPanel(
                        username = state.regUsername,
                        onUsernameChange = viewModel::updateRegUsername,
                        email = state.regEmail,
                        onEmailChange = viewModel::updateRegEmail,
                        password = state.regPassword,
                        onPasswordChange = viewModel::updateRegPassword,
                        passwordVisible = state.regPasswordVisible,
                        onTogglePasswordVisibility = viewModel::toggleRegPasswordVisibility,
                        agreeTerms = state.agreeTerms,
                        onAgreeTermsChange = viewModel::updateAgreeTerms,
                        isLoading = state.isLoading,
                        onRegister = {
                            focusManager.clearFocus()
                            viewModel.register(onSuccess = onNavigateToChannels)
                        }
                    )
                }
            }

            Spacer(modifier = Modifier.height(16.dp))
        }

        // Bottom terms
        TermsFooter()
    }
}

@Composable
private fun LogoSection(isLogin: Boolean) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .size(72.dp)
                .clip(RoundedCornerShape(22.dp))
                .background(
                    Brush.linearGradient(listOf(AccentPurple, AccentPink))
                ),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                painter = painterResource(R.drawable.ic_logo_white),
                contentDescription = "Capella",
                tint = Color.Unspecified,
                modifier = Modifier.size(52.dp)
            )
        }

        Spacer(modifier = Modifier.height(16.dp))

        Text(
            text = if (isLogin) "欢迎回来" else "创建账号",
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            color = Foreground
        )

        Spacer(modifier = Modifier.height(8.dp))

        Text(
            text = if (isLogin) "登录你的 Capella 账号" else "开始你的 Capella 之旅",
            style = MaterialTheme.typography.bodyLarge,
            color = Muted
        )
    }
}

@Composable
private fun TabSwitcher(
    isLogin: Boolean,
    onSelectLogin: () -> Unit,
    onSelectRegister: () -> Unit
) {
    Surface(
        shape = MaterialTheme.shapes.small,
        color = Surface
    ) {
        Row(
            modifier = Modifier.fillMaxWidth()
        ) {
            // Login tab
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clip(MaterialTheme.shapes.small)
                    .background(if (isLogin) SurfaceElevated else Color.Transparent)
                    .clickable { onSelectLogin() }
                    .padding(vertical = 12.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "登录",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = if (isLogin) Foreground else Muted
                )
            }

            // Register tab
            Box(
                modifier = Modifier
                    .weight(1f)
                    .clip(MaterialTheme.shapes.small)
                    .background(if (!isLogin) SurfaceElevated else Color.Transparent)
                    .clickable { onSelectRegister() }
                    .padding(vertical = 12.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "注册",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = if (!isLogin) Foreground else Muted
                )
            }
        }
    }
}

@Composable
private fun LoginPanel(
    email: String,
    onEmailChange: (String) -> Unit,
    password: String,
    onPasswordChange: (String) -> Unit,
    passwordVisible: Boolean,
    onTogglePasswordVisibility: () -> Unit,
    rememberMe: Boolean,
    onRememberMeChange: (Boolean) -> Unit,
    isLoading: Boolean,
    onLogin: () -> Unit
) {
    Column {
        // Email
        FormLabel(text = "邮箱")
        Spacer(modifier = Modifier.height(8.dp))
        CapellaTextField(
            value = email,
            onValueChange = onEmailChange,
            placeholder = "you@example.com",
            keyboardType = KeyboardType.Email,
            imeAction = ImeAction.Next,
            singleLine = true
        )

        Spacer(modifier = Modifier.height(20.dp))

        // Password
        FormLabel(text = "密码")
        Spacer(modifier = Modifier.height(8.dp))
        CapellaTextField(
            value = password,
            onValueChange = onPasswordChange,
            placeholder = "输入密码",
            keyboardType = KeyboardType.Password,
            imeAction = ImeAction.Done,
            singleLine = true,
            visualTransformation = if (passwordVisible) VisualTransformation.None
                else PasswordVisualTransformation(),
            trailingIcon = {
                IconButton(onClick = onTogglePasswordVisibility) {
                    Icon(
                        imageVector = if (passwordVisible) Icons.Default.VisibilityOff
                            else Icons.Default.Visibility,
                        contentDescription = if (passwordVisible) "隐藏密码" else "显示密码",
                        tint = Muted
                    )
                }
            },
            onKeyboardAction = { if (!isLoading) onLogin() }
        )

        Spacer(modifier = Modifier.height(16.dp))

        // Options row
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Remember me
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.clickable { onRememberMeChange(!rememberMe) }
            ) {
                Checkbox(
                    checked = rememberMe,
                    onCheckedChange = onRememberMeChange,
                    colors = CheckboxDefaults.colors(
                        checkedColor = AccentPurple,
                        uncheckedColor = Muted,
                        checkmarkColor = Color.White
                    )
                )
                Spacer(modifier = Modifier.width(4.dp))
                Text(
                    text = "记住我",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Muted
                )
            }

            Text(
                text = "忘记密码？",
                style = MaterialTheme.typography.bodyMedium,
                color = AccentPurple,
                fontWeight = FontWeight.Medium
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Login button
        Button(
            onClick = onLogin,
            enabled = !isLoading,
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = AccentPurple,
                disabledContainerColor = AccentPurple.copy(alpha = 0.6f)
            ),
            shape = MaterialTheme.shapes.extraLarge
        ) {
            if (isLoading) {
                Box(
                    modifier = Modifier
                        .size(20.dp)
                        .clip(MaterialTheme.shapes.extraLarge)
                        .background(Color.White.copy(alpha = 0.3f))
                )
                // In a real app, use a circular progress indicator here
            }
            Text(
                text = "登录",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Divider
        DividerWithText(text = "或使用以下方式")

        Spacer(modifier = Modifier.height(24.dp))

        // Social login
        SocialLoginButtons()
    }
}

@Composable
private fun RegisterPanel(
    username: String,
    onUsernameChange: (String) -> Unit,
    email: String,
    onEmailChange: (String) -> Unit,
    password: String,
    onPasswordChange: (String) -> Unit,
    passwordVisible: Boolean,
    onTogglePasswordVisibility: () -> Unit,
    agreeTerms: Boolean,
    onAgreeTermsChange: (Boolean) -> Unit,
    isLoading: Boolean,
    onRegister: () -> Unit
) {
    Column {
        // Username
        FormLabel(text = "用户名")
        Spacer(modifier = Modifier.height(8.dp))
        CapellaTextField(
            value = username,
            onValueChange = onUsernameChange,
            placeholder = "设置你的用户名",
            keyboardType = KeyboardType.Text,
            imeAction = ImeAction.Next,
            singleLine = true
        )

        Spacer(modifier = Modifier.height(20.dp))

        // Email
        FormLabel(text = "邮箱")
        Spacer(modifier = Modifier.height(8.dp))
        CapellaTextField(
            value = email,
            onValueChange = onEmailChange,
            placeholder = "you@example.com",
            keyboardType = KeyboardType.Email,
            imeAction = ImeAction.Next,
            singleLine = true
        )

        Spacer(modifier = Modifier.height(20.dp))

        // Password
        FormLabel(text = "密码")
        Spacer(modifier = Modifier.height(8.dp))
        CapellaTextField(
            value = password,
            onValueChange = onPasswordChange,
            placeholder = "设置密码（至少8位）",
            keyboardType = KeyboardType.Password,
            imeAction = ImeAction.Done,
            singleLine = true,
            visualTransformation = if (passwordVisible) VisualTransformation.None
                else PasswordVisualTransformation(),
            trailingIcon = {
                IconButton(onClick = onTogglePasswordVisibility) {
                    Icon(
                        imageVector = if (passwordVisible) Icons.Default.VisibilityOff
                            else Icons.Default.Visibility,
                        contentDescription = if (passwordVisible) "隐藏密码" else "显示密码",
                        tint = Muted
                    )
                }
            },
            onKeyboardAction = { if (!isLoading) onRegister() }
        )

        Spacer(modifier = Modifier.height(16.dp))

        // Terms checkbox
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.clickable { onAgreeTermsChange(!agreeTerms) }
        ) {
            Checkbox(
                checked = agreeTerms,
                onCheckedChange = onAgreeTermsChange,
                colors = CheckboxDefaults.colors(
                    checkedColor = AccentPurple,
                    uncheckedColor = Muted,
                    checkmarkColor = Color.White
                )
            )
            Spacer(modifier = Modifier.width(4.dp))
            val termsText = buildAnnotatedString {
                append("我同意 ")
                withStyle(SpanStyle(color = AccentPurple)) {
                    append("服务条款")
                }
                append(" 和 ")
                withStyle(SpanStyle(color = AccentPurple)) {
                    append("隐私政策")
                }
            }
            Text(
                text = termsText,
                style = MaterialTheme.typography.bodyMedium,
                color = Muted
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Register button
        Button(
            onClick = onRegister,
            enabled = !isLoading,
            modifier = Modifier
                .fillMaxWidth()
                .height(52.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = AccentPurple,
                disabledContainerColor = AccentPurple.copy(alpha = 0.6f)
            ),
            shape = MaterialTheme.shapes.extraLarge
        ) {
            Text(
                text = "创建账号",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Divider
        DividerWithText(text = "或使用以下方式")

        Spacer(modifier = Modifier.height(24.dp))

        // Social login
        SocialLoginButtons()
    }
}

@Composable
private fun FormLabel(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelLarge,
        color = Muted
    )
}

@Composable
private fun CapellaTextField(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    keyboardType: KeyboardType,
    imeAction: ImeAction,
    singleLine: Boolean,
    visualTransformation: VisualTransformation = VisualTransformation.None,
    trailingIcon: @Composable (() -> Unit)? = null,
    onKeyboardAction: () -> Unit = {}
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        placeholder = {
            Text(
                text = placeholder,
                color = Muted.copy(alpha = 0.6f)
            )
        },
        singleLine = singleLine,
        visualTransformation = visualTransformation,
        trailingIcon = trailingIcon,
        keyboardOptions = KeyboardOptions(
            keyboardType = keyboardType,
            imeAction = imeAction
        ),
        keyboardActions = KeyboardActions(
            onDone = { onKeyboardAction() },
            onNext = { onKeyboardAction() }
        ),
        modifier = Modifier.fillMaxWidth(),
        textStyle = MaterialTheme.typography.bodyLarge.copy(
            color = Foreground
        ),
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = AccentPurple,
            unfocusedBorderColor = Border,
            cursorColor = AccentPurple,
            focusedContainerColor = Surface,
            unfocusedContainerColor = Surface
        ),
        shape = MaterialTheme.shapes.small
    )
}

@Composable
private fun DividerWithText(text: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier = Modifier
                .weight(1f)
                .height(1.dp)
                .background(Border)
        )
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 16.dp),
            style = MaterialTheme.typography.labelMedium,
            color = Muted
        )
        Box(
            modifier = Modifier
                .weight(1f)
                .height(1.dp)
                .background(Border)
        )
    }
}

@Composable
private fun SocialLoginButtons() {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        // Google
        Surface(
            modifier = Modifier.weight(1f),
            shape = MaterialTheme.shapes.extraLarge,
            color = Surface,
            border = androidx.compose.foundation.BorderStroke(1.dp, Border)
        ) {
            Row(
                modifier = Modifier
                    .clickable { /* TODO: Google login */ }
                    .padding(vertical = 12.dp),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    painter = painterResource(R.drawable.ic_google),
                    contentDescription = "Google 登录",
                    modifier = Modifier.size(20.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = "Google",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Foreground
                )
            }
        }

        // GitHub
        Surface(
            modifier = Modifier.weight(1f),
            shape = MaterialTheme.shapes.extraLarge,
            color = Surface,
            border = androidx.compose.foundation.BorderStroke(1.dp, Border)
        ) {
            Row(
                modifier = Modifier
                    .clickable { /* TODO: GitHub login */ }
                    .padding(vertical = 12.dp),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    painter = painterResource(R.drawable.ic_github),
                    contentDescription = "GitHub 登录",
                    modifier = Modifier.size(20.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = "GitHub",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Foreground
                )
            }
        }
    }
}

@Composable
private fun TermsFooter() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .background(Background)
            .padding(horizontal = 24.dp)
            .padding(bottom = 32.dp)
    ) {
        Text(
            text = buildAnnotatedString {
                append("继续使用即表示你同意我们的\n")
                withStyle(SpanStyle(color = AccentPurple)) {
                    append("服务条款")
                }
                append(" 和 ")
                withStyle(SpanStyle(color = AccentPurple)) {
                    append("隐私政策")
                }
            },
            style = MaterialTheme.typography.labelSmall,
            color = Muted,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth()
        )
    }
}
