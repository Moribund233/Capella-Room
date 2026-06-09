package com.capella.room.di

import com.capella.room.data.local.TokenManager
import com.capella.room.data.remote.api.AuthApi
import com.capella.room.data.remote.api.FileApi
import com.capella.room.data.remote.api.RoomApi
import com.capella.room.data.remote.api.UserApi
import com.capella.room.data.remote.dto.RefreshRequest
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.runBlocking
import okhttp3.Authenticator
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.Route
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.moshi.MoshiConverterFactory
import java.util.concurrent.TimeUnit
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object NetworkModule {

    // 生产环境服务器 (Cloudflare Tunnel)
    private const val BASE_URL = "https://chat.moribund.top/"
    // 本地开发环境
    // private const val BASE_URL = "http://10.0.2.2:3000/"

    @Provides
    @Singleton
    fun provideMoshi(): Moshi = Moshi.Builder()
        .add(KotlinJsonAdapterFactory())
        .build()

    @Provides
    @Singleton
    fun provideOkHttpClient(tokenManager: TokenManager, moshi: Moshi): OkHttpClient {
        val authInterceptor = Interceptor { chain ->
            val request = chain.request()
            // Only add auth header if a token exists
            val token = runBlocking { tokenManager.getAccessToken() }
            if (token != null) {
                chain.proceed(request.newBuilder()
                    .addHeader("Authorization", "Bearer $token")
                    .build())
            } else {
                chain.proceed(request)
            }
        }

        val logging = HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BODY
        }

        // 创建一个独立的 Retrofit 用于 Token 刷新（避免循环依赖）
        val refreshClient = OkHttpClient.Builder()
            .addInterceptor(logging)
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .build()

        val refreshRetrofit = Retrofit.Builder()
            .baseUrl(BASE_URL)
            .client(refreshClient)
            .addConverterFactory(MoshiConverterFactory.create(moshi))
            .build()

        val authApi = refreshRetrofit.create(AuthApi::class.java)

        // Token 刷新 Authenticator - 处理 401 错误
        val tokenAuthenticator = object : Authenticator {
            override fun authenticate(route: Route?, response: Response): Request? {
                // 避免无限重试
                if (response.request.header("X-Retry-Auth") != null) {
                    return null
                }

                val refreshToken = runBlocking { tokenManager.getRefreshToken() }
                    ?: return null

                return runBlocking {
                    try {
                        val refreshResponse = authApi.refresh(RefreshRequest(refreshToken))
                        if (refreshResponse.isSuccessful && refreshResponse.body()?.success == true) {
                            val newToken = refreshResponse.body()?.data
                            if (newToken != null) {
                                tokenManager.saveTokens(newToken.accessToken, newToken.refreshToken)
                                // 使用新 Token 重试原请求
                                response.request.newBuilder()
                                    .header("Authorization", "Bearer ${newToken.accessToken}")
                                    .header("X-Retry-Auth", "true")
                                    .build()
                            } else {
                                null
                            }
                        } else {
                            // 刷新失败，清除 Token
                            tokenManager.clear()
                            null
                        }
                    } catch (e: Exception) {
                        tokenManager.clear()
                        null
                    }
                }
            }
        }

        return OkHttpClient.Builder()
            .addInterceptor(authInterceptor)
            .addInterceptor(logging)
            .authenticator(tokenAuthenticator)
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .build()
    }

    @Provides
    @Singleton
    fun provideRetrofit(moshi: Moshi, client: OkHttpClient): Retrofit =
        Retrofit.Builder()
            .baseUrl(BASE_URL)
            .client(client)
            .addConverterFactory(MoshiConverterFactory.create(moshi))
            .build()

    @Provides
    @Singleton
    fun provideAuthApi(retrofit: Retrofit): AuthApi =
        retrofit.create(AuthApi::class.java)

    @Provides
    @Singleton
    fun provideUserApi(retrofit: Retrofit): UserApi =
        retrofit.create(UserApi::class.java)

    @Provides
    @Singleton
    fun provideRoomApi(retrofit: Retrofit): RoomApi =
        retrofit.create(RoomApi::class.java)

    @Provides
    @Singleton
    fun provideFileApi(retrofit: Retrofit): FileApi =
        retrofit.create(FileApi::class.java)
}
