package com.capella.room.data.remote.dto

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

/**
 * 文件 DTO
 */
@JsonClass(generateAdapter = true)
data class FileDto(
    @Json(name = "id") val id: String,
    @Json(name = "filename") val filename: String,
    @Json(name = "original_name") val originalName: String,
    @Json(name = "mime_type") val mimeType: String,
    @Json(name = "size") val size: Long,
    @Json(name = "url") val url: String,
    @Json(name = "thumbnail_url") val thumbnailUrl: String? = null,
    @Json(name = "uploader") val uploader: UserInfo? = null,
    @Json(name = "room_id") val roomId: String? = null,
    @Json(name = "created_at") val createdAt: String = ""
)

/**
 * 文件上传请求
 */
@JsonClass(generateAdapter = true)
data class FileUploadRequest(
    @Json(name = "filename") val filename: String,
    @Json(name = "content_type") val contentType: String,
    @Json(name = "size") val size: Long
)
