package com.capella.room.ui.navigation

sealed class Screen(val route: String) {
    data object Splash : Screen("splash")
    data object Login : Screen("login")
    data object Channels : Screen("channels")
    data object Chat : Screen("chat/{channelId}") {
        fun createRoute(channelId: String) = "chat/$channelId"
    }
    data object Profile : Screen("profile")
}
