package com.capella.room.data.remote.api

import com.capella.room.data.remote.dto.*
import retrofit2.Response
import retrofit2.http.*

interface UserApi {

    // ── Profile ──

    @GET("api/v1/users/me")
    suspend fun getProfile(): Response<ApiResponse<UserDto>>

    @PUT("api/v1/users/me")
    suspend fun updateProfile(@Body body: Map<String, @JvmSuppressWildcards Any>): Response<ApiResponse<UserDto>>

    @PUT("api/v1/users/me/password")
    suspend fun changePassword(@Body body: Map<String, String>): Response<ApiResponse<Unit>>

    @POST("api/v1/users/logout")
    suspend fun logout(): Response<ApiResponse<Unit>>

    // ── My Rooms ──

    @GET("api/v1/users/me/rooms")
    suspend fun getMyRooms(): Response<ApiResponse<List<RoomDto>>>

    // ── User search / lookup ──

    @GET("api/v1/users")
    suspend fun getUsers(
        @Query("search") search: String? = null,
        @Query("limit") limit: Int = 20,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<List<UserDto>>>

    @GET("api/v1/users/search")
    suspend fun searchUsers(
        @Query("q") query: String,
        @Query("limit") limit: Int = 20,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<List<UserDto>>>

    @GET("api/v1/users/{user_id}")
    suspend fun getUserById(@Path("user_id") userId: String): Response<ApiResponse<UserDto>>

    // ── Settings ──

    @GET("api/v1/users/me/settings")
    suspend fun getSettings(): Response<ApiResponse<Map<String, @JvmSuppressWildcards Any>>>

    @PATCH("api/v1/users/me/settings")
    suspend fun updateSettings(@Body body: Map<String, @JvmSuppressWildcards Any>): Response<ApiResponse<Map<String, @JvmSuppressWildcards Any>>>

    // ── Friends ──

    @GET("api/v1/users/friends")
    suspend fun getFriends(): Response<ApiResponse<List<UserDto>>>

    @POST("api/v1/users/friends/requests")
    suspend fun sendFriendRequest(@Body body: Map<String, String>): Response<ApiResponse<Unit>>

    @GET("api/v1/users/friends/requests/received")
    suspend fun getReceivedFriendRequests(): Response<ApiResponse<List<Any>>>

    @GET("api/v1/users/friends/requests/sent")
    suspend fun getSentFriendRequests(): Response<ApiResponse<List<Any>>>

    @POST("api/v1/users/friends/requests/handle")
    suspend fun handleFriendRequest(@Body body: Map<String, String>): Response<ApiResponse<Unit>>

    @DELETE("api/v1/users/friends/requests/{id}")
    suspend fun cancelFriendRequest(@Path("id") requestId: String): Response<ApiResponse<Unit>>

    @DELETE("api/v1/users/friends/{id}")
    suspend fun removeFriend(@Path("id") friendId: String): Response<ApiResponse<Unit>>

    // ── Security ──

    @GET("api/v1/users/me/security/overview")
    suspend fun getSecurityOverview(): Response<ApiResponse<Map<String, @JvmSuppressWildcards Any>>>

    @GET("api/v1/users/me/devices")
    suspend fun getDevices(): Response<ApiResponse<List<Any>>>

    @DELETE("api/v1/users/me/devices/{device_id}")
    suspend fun logoutDevice(@Path("device_id") deviceId: String): Response<ApiResponse<Unit>>

    @GET("api/v1/users/me/login-history")
    suspend fun getLoginHistory(): Response<ApiResponse<List<Any>>>
}
