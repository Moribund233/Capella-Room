package com.capella.room.ui.screen.splash

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.local.TokenManager
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class SplashViewModel @Inject constructor(
    private val tokenManager: TokenManager
) : ViewModel() {

    fun checkLoginState(onResult: (isLoggedIn: Boolean) -> Unit) {
        viewModelScope.launch {
            val loggedIn = tokenManager.isLoggedIn.first()
            onResult(loggedIn)
        }
    }
}
