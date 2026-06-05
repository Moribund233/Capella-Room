package com.capella.room.data.remote.api

import com.capella.room.data.remote.dto.ApiResponse
import com.capella.room.data.remote.dto.FileDto
import okhttp3.MultipartBody
import retrofit2.Response
import retrofit2.http.*

/**
 * 文件上传 API
 */
interface FileApi {

    /**
     * 通用文件上传
     */
    @Multipart
    @POST("api/v1/upload")
    suspend fun uploadFile(
        @Part file: MultipartBody.Part
    ): Response<ApiResponse<FileDto>>

    /**
     * 上传图片
     */
    @Multipart
    @POST("api/v1/upload/image")
    suspend fun uploadImage(
        @Part image: MultipartBody.Part
    ): Response<ApiResponse<FileDto>>

    /**
     * 上传头像
     */
    @Multipart
    @POST("api/v1/upload/avatar")
    suspend fun uploadAvatar(
        @Part avatar: MultipartBody.Part
    ): Response<ApiResponse<FileDto>>

    /**
     * 获取文件列表
     */
    @GET("api/v1/files")
    suspend fun getFiles(
        @Query("room_id") roomId: String? = null,
        @Query("type") type: String? = null,
        @Query("limit") limit: Int = 50,
        @Query("offset") offset: Int = 0
    ): Response<ApiResponse<List<FileDto>>>

    /**
     * 获取文件详情
     */
    @GET("api/v1/files/{file_id}")
    suspend fun getFile(
        @Path("file_id") fileId: String
    ): Response<ApiResponse<FileDto>>

    /**
     * 删除文件
     */
    @DELETE("api/v1/files/{file_id}")
    suspend fun deleteFile(
        @Path("file_id") fileId: String
    ): Response<ApiResponse<Unit>>
}
