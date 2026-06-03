package com.capella.room.ui.theme

import android.app.Activity
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

private val CapellaColorScheme = darkColorScheme(
    primary = AccentPurple,
    onPrimary = Foreground,
    primaryContainer = AccentPurpleSoft,
    onPrimaryContainer = AccentPurple,

    secondary = AccentPink,
    onSecondary = Foreground,

    tertiary = AccentBlue,
    onTertiary = Foreground,

    background = Background,
    onBackground = Foreground,
    surface = Surface,
    onSurface = Foreground,
    surfaceVariant = SurfaceElevated,
    onSurfaceVariant = Muted,
    outline = Border,
    outlineVariant = Border,

    error = AccentOrange,
    onError = Foreground
)

@Composable
fun CapellaRoomTheme(
    content: @Composable () -> Unit
) {
    val colorScheme = CapellaColorScheme
    val view = LocalView.current

    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            window.statusBarColor = colorScheme.background.toArgb()
            window.navigationBarColor = colorScheme.background.toArgb()
            WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = false
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = CapellaTypography,
        content = content
    )
}
