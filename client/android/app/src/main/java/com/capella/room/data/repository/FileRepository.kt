package com.capella.room.data.repository

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import com.capella.room.data.remote.api.FileApi
import com.capella.room.data.remote.dto.FileDto
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.asRequestBody
import java.io.File
import java.io.FileOutputStream
import javax.inject.Inject
import javax.inject.Singleton

/**
 * 文件上传仓库
 */
@Singleton
class FileRepository @Inject constructor(
    private val fileApi: FileApi,
    @ApplicationContext private val context: Context
) {

    /**
     * 上传图片
     */
    suspend fun uploadImage(uri: Uri): Result<FileDto> = withContext(Dispatchers.IO) {
        try {
            val file = uriToFile(uri) ?: return@withContext Result.failure(Exception("Failed to create file from URI"))
            val mimeType = context.contentResolver.getType(uri) ?: "image/jpeg"

            val requestBody = file.asRequestBody(mimeType.toMediaTypeOrNull())
            val multipartBody = MultipartBody.Part.createFormData(
                "image",
                file.name,
                requestBody
            )

            val response = fileApi.uploadImage(multipartBody)

            file.delete() // 清理临时文件

            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception(response.errorBody()?.string() ?: "Upload failed"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * 上传文件
     */
    suspend fun uploadFile(uri: Uri): Result<FileDto> = withContext(Dispatchers.IO) {
        try {
            val file = uriToFile(uri) ?: return@withContext Result.failure(Exception("Failed to create file from URI"))
            val mimeType = context.contentResolver.getType(uri) ?: "application/octet-stream"

            val requestBody = file.asRequestBody(mimeType.toMediaTypeOrNull())
            val multipartBody = MultipartBody.Part.createFormData(
                "file",
                file.name,
                requestBody
            )

            val response = fileApi.uploadFile(multipartBody)

            file.delete() // 清理临时文件

            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception(response.errorBody()?.string() ?: "Upload failed"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * 上传头像
     */
    suspend fun uploadAvatar(uri: Uri): Result<FileDto> = withContext(Dispatchers.IO) {
        try {
            val file = uriToFile(uri) ?: return@withContext Result.failure(Exception("Failed to create file from URI"))
            val mimeType = context.contentResolver.getType(uri) ?: "image/jpeg"

            val requestBody = file.asRequestBody(mimeType.toMediaTypeOrNull())
            val multipartBody = MultipartBody.Part.createFormData(
                "avatar",
                file.name,
                requestBody
            )

            val response = fileApi.uploadAvatar(multipartBody)

            file.delete() // 清理临时文件

            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data?.let {
                    Result.success(it)
                } ?: Result.failure(Exception("Empty response"))
            } else {
                Result.failure(Exception(response.errorBody()?.string() ?: "Upload failed"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * 获取文件列表
     */
    suspend fun getFiles(roomId: String? = null, type: String? = null): List<FileDto> {
        return try {
            val response = fileApi.getFiles(roomId = roomId, type = type)
            if (response.isSuccessful && response.body()?.success == true) {
                response.body()?.data ?: emptyList()
            } else {
                emptyList()
            }
        } catch (e: Exception) {
            emptyList()
        }
    }

    /**
     * 删除文件
     */
    suspend fun deleteFile(fileId: String): Boolean {
        return try {
            val response = fileApi.deleteFile(fileId)
            response.isSuccessful && response.body()?.success == true
        } catch (e: Exception) {
            false
        }
    }

    /**
     * 将 URI 转换为临时文件
     */
    private fun uriToFile(uri: Uri): File? {
        return try {
            val contentResolver = context.contentResolver
            val fileName = getFileName(uri) ?: "temp_file"
            val tempFile = File(context.cacheDir, fileName)

            val inputStream = contentResolver.openInputStream(uri) ?: return null
            inputStream.use { input ->
                FileOutputStream(tempFile).use { output ->
                    input.copyTo(output)
                }
            }

            tempFile
        } catch (e: Exception) {
            null
        }
    }

    /**
     * 获取文件名
     */
    private fun getFileName(uri: Uri): String? {
        var result: String? = null
        if (uri.scheme == "content") {
            context.contentResolver.query(uri, null, null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (index >= 0) {
                        result = cursor.getString(index)
                    }
                }
            }
        }
        if (result == null) {
            result = uri.path
            val cut = result?.lastIndexOf('/')
            if (cut != -1) {
                result = result?.substring(cut!! + 1)
            }
        }
        return result
    }

    /**
     * 格式化文件大小
     */
    fun formatFileSize(size: Long): String {
        return when {
            size < 1024 -> "$size B"
            size < 1024 * 1024 -> "${size / 1024} KB"
            size < 1024 * 1024 * 1024 -> "${size / (1024 * 1024)} MB"
            else -> "${size / (1024 * 1024 * 1024)} GB"
        }
    }

    /**
     * 检查是否为图片类型
     */
    fun isImage(mimeType: String): Boolean {
        return mimeType.startsWith("image/")
    }

    /**
     * 检查文件大小是否超过限制（默认 50MB）
     */
    fun isFileSizeValid(size: Long, maxSize: Long = 50 * 1024 * 1024): Boolean {
        return size <= maxSize
    }
}
