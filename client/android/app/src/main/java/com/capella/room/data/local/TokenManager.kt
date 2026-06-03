package com.capella.room.data.local

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

private val Context.tokenStore: DataStore<Preferences> by preferencesDataStore(name = "auth_tokens")

@Singleton
class TokenManager @Inject constructor(
    @ApplicationContext private val context: Context
) {
    companion object {
        private val ACCESS_TOKEN = stringPreferencesKey("access_token")
        private val REFRESH_TOKEN = stringPreferencesKey("refresh_token")
        private val USER_ID = stringPreferencesKey("user_id")
        private val USERNAME = stringPreferencesKey("username")
    }

    val accessTokenFlow: Flow<String?> = context.tokenStore.data.map { it[ACCESS_TOKEN] }
    val refreshTokenFlow: Flow<String?> = context.tokenStore.data.map { it[REFRESH_TOKEN] }

    suspend fun getAccessToken(): String? = context.tokenStore.data.first()[ACCESS_TOKEN]

    suspend fun getRefreshToken(): String? = context.tokenStore.data.first()[REFRESH_TOKEN]

    suspend fun saveTokens(accessToken: String, refreshToken: String) {
        context.tokenStore.edit {
            it[ACCESS_TOKEN] = accessToken
            it[REFRESH_TOKEN] = refreshToken
        }
    }

    suspend fun saveUserInfo(userId: String, username: String) {
        context.tokenStore.edit {
            it[USER_ID] = userId
            it[USERNAME] = username
        }
    }

    suspend fun clear() {
        context.tokenStore.edit { it.clear() }
    }

    val isLoggedIn: Flow<Boolean> = accessTokenFlow.map { !it.isNullOrBlank() }
}
