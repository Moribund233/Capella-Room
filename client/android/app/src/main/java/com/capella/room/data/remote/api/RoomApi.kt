package com.capella.room.data.remote.api

import com.capella.room.data.remote.dto.*
import retrofit2.Response
import retrofit2.http.*

interface RoomApi {

    @GET("api/v1/rooms")
    suspend fun getRooms(
        @Query("search") search: String? = null,
        @Query("limit") limit: Int = 50,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<PaginatedData<RoomDto>>>

    @GET("api/v1/rooms/recent")
    suspend fun getRecentRooms(): Response<ApiResponse<List<RoomDto>>>

    @POST("api/v1/rooms")
    suspend fun createRoom(@Body request: CreateRoomRequest): Response<ApiResponse<RoomDto>>

    @GET("api/v1/rooms/{room_id}")
    suspend fun getRoom(@Path("room_id") roomId: String): Response<ApiResponse<RoomDto>>

    @DELETE("api/v1/rooms/{room_id}")
    suspend fun deleteRoom(@Path("room_id") roomId: String): Response<ApiResponse<Unit>>

    @POST("api/v1/rooms/{room_id}/join")
    suspend fun joinRoom(@Path("room_id") roomId: String): Response<ApiResponse<Unit>>

    @DELETE("api/v1/rooms/{room_id}/leave")
    suspend fun leaveRoom(@Path("room_id") roomId: String): Response<ApiResponse<Unit>>

    @GET("api/v1/rooms/{room_id}/members")
    suspend fun getMembers(@Path("room_id") roomId: String): Response<ApiResponse<List<Any>>>

    @GET("api/v1/rooms/{room_id}/messages")
    suspend fun getMessages(
        @Path("room_id") roomId: String,
        @Query("limit") limit: Int = 50,
        @Query("before") before: String? = null
    ): Response<ApiResponse<MessageListDto>>
}
