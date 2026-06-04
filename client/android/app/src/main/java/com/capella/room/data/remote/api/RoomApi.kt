package com.capella.room.data.remote.api

import com.capella.room.data.remote.dto.*
import retrofit2.Response
import retrofit2.http.*

interface RoomApi {

    // ── Room CRUD ──

    @GET("api/v1/rooms")
    suspend fun getRooms(
        @Query("search") search: String? = null,
        @Query("limit") limit: Int = 50,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<List<RoomDto>>>

    @GET("api/v1/rooms/recent")
    suspend fun getRecentRooms(
        @Query("limit") limit: Int = 20,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<List<RoomDto>>>

    @POST("api/v1/rooms")
    suspend fun createRoom(@Body request: CreateRoomRequest): Response<ApiResponse<RoomDto>>

    @PUT("api/v1/rooms/{room_id}")
    suspend fun updateRoom(
        @Path("room_id") roomId: String,
        @Body body: Map<String, @JvmSuppressWildcards Any>
    ): Response<ApiResponse<RoomDto>>

    @GET("api/v1/rooms/{room_id}")
    suspend fun getRoom(@Path("room_id") roomId: String): Response<ApiResponse<RoomDto>>

    @DELETE("api/v1/rooms/{room_id}")
    suspend fun deleteRoom(@Path("room_id") roomId: String): Response<ApiResponse<Unit>>

    // ── Direct / DM rooms ──

    @POST("api/v1/rooms/direct")
    suspend fun createOrGetDirectRoom(@Body request: DirectRoomRequest): Response<ApiResponse<DirectRoomDto>>

    @GET("api/v1/rooms/direct/list")
    suspend fun getDirectRoomList(): Response<ApiResponse<List<DirectRoomDto>>>

    // ── Join / Leave ──

    @POST("api/v1/rooms/{room_id}/join")
    suspend fun joinRoom(@Path("room_id") roomId: String): Response<ApiResponse<Unit>>

    @DELETE("api/v1/rooms/{room_id}/leave")
    suspend fun leaveRoom(@Path("room_id") roomId: String): Response<ApiResponse<Unit>>

    // ── Members ──

    @GET("api/v1/rooms/{room_id}/members")
    suspend fun getMembers(@Path("room_id") roomId: String): Response<ApiResponse<List<RoomMemberDto>>>

    @DELETE("api/v1/rooms/{room_id}/members/{user_id}")
    suspend fun kickMember(
        @Path("room_id") roomId: String,
        @Path("user_id") userId: String
    ): Response<ApiResponse<Unit>>

    @PUT("api/v1/rooms/{room_id}/members/{user_id}/role")
    suspend fun setMemberRole(
        @Path("room_id") roomId: String,
        @Path("user_id") userId: String,
        @Body request: UpdateRoleRequest
    ): Response<ApiResponse<Unit>>

    // ── Invitations ──

    @GET("api/v1/rooms/{room_id}/invitations")
    suspend fun getInvitations(@Path("room_id") roomId: String): Response<ApiResponse<List<InvitationDto>>>

    @POST("api/v1/rooms/{room_id}/invitations")
    suspend fun createInvitation(
        @Path("room_id") roomId: String,
        @Body request: CreateInvitationRequest
    ): Response<ApiResponse<InvitationDto>>

    @DELETE("api/v1/rooms/{room_id}/invitations/{invitation_id}")
    suspend fun revokeInvitation(
        @Path("room_id") roomId: String,
        @Path("invitation_id") invitationId: String
    ): Response<ApiResponse<Unit>>

    @POST("api/v1/rooms/join-by-invite")
    suspend fun joinByInvite(@Body request: JoinByInviteRequest): Response<ApiResponse<Unit>>

    @GET("api/v1/rooms/validate-invite")
    suspend fun validateInvite(@Query("invite_code") inviteCode: String): Response<ApiResponse<InviteValidationDto>>

    // ── Messages ──

    @GET("api/v1/rooms/{room_id}/messages")
    suspend fun getMessages(
        @Path("room_id") roomId: String,
        @Query("limit") limit: Int = 50,
        @Query("before") before: String? = null
    ): Response<ApiResponse<MessageListDto>>

    @GET("api/v1/messages/search")
    suspend fun searchMessages(
        @Query("q") query: String,
        @Query("room_id") roomId: String? = null,
        @Query("limit") limit: Int = 50,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<List<MessageDto>>>

    @PUT("api/v1/messages/{message_id}")
    suspend fun editMessage(
        @Path("message_id") messageId: String,
        @Body body: Map<String, String>
    ): Response<ApiResponse<MessageDto>>

    @DELETE("api/v1/messages/{message_id}")
    suspend fun deleteMessage(@Path("message_id") messageId: String): Response<ApiResponse<Unit>>

    @GET("api/v1/messages/{message_id}/history")
    suspend fun getMessageHistory(@Path("message_id") messageId: String): Response<ApiResponse<List<MessageDto>>>
}
