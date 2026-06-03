package com.capella.room.data.remote.api

import com.capella.room.data.remote.dto.UserDto
import com.capella.room.data.remote.dto.ApiResponse
import retrofit2.Response
import retrofit2.http.GET
import retrofit2.http.PUT
import retrofit2.http.Body

interface UserApi {

    @GET("api/v1/users/me")
    suspend fun getProfile(): Response<ApiResponse<UserDto>>

    @PUT("api/v1/users/me")
    suspend fun updateProfile(@Body body: Map<String, String>): Response<ApiResponse<UserDto>>

    @GET("api/v1/users/me/rooms")
    suspend fun getMyRooms(): Response<ApiResponse<List<Any>>>
}
