package com.capella.room.ui.screen.room

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.widget.Toast
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.capella.room.data.remote.api.RoomApi
import com.capella.room.data.remote.dto.CreateInvitationRequest
import com.capella.room.data.remote.dto.InvitationDto
import com.capella.room.data.remote.dto.JoinByInviteRequest
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import javax.inject.Inject

/**
 * 房间邀请管理 ViewModel
 */
@HiltViewModel
class RoomInvitationViewModel @Inject constructor(
    private val roomApi: RoomApi
) : ViewModel() {

    /**
     * 获取房间的邀请列表
     */
    suspend fun getInvitations(roomId: String): List<InvitationDto> {
        return try {
            val response = roomApi.getInvitations(roomId)
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
     * 创建邀请链接
     */
    fun createInvitation(
        roomId: String,
        expiresInHours: Int,
        onSuccess: (InvitationDto) -> Unit,
        onError: (String) -> Unit = {}
    ) {
        viewModelScope.launch {
            try {
                val request = CreateInvitationRequest(
                    maxUses = 10,
                    expiresInHours = expiresInHours
                )
                val response = roomApi.createInvitation(roomId, request)
                if (response.isSuccessful && response.body()?.success == true) {
                    response.body()?.data?.let { onSuccess(it) }
                } else {
                    onError("创建邀请失败")
                }
            } catch (e: Exception) {
                onError(e.message ?: "创建邀请失败")
            }
        }
    }

    /**
     * 撤销邀请链接
     */
    fun revokeInvitation(
        roomId: String,
        invitationId: String,
        onSuccess: () -> Unit = {},
        onError: (String) -> Unit = {}
    ) {
        viewModelScope.launch {
            try {
                val response = roomApi.revokeInvitation(roomId, invitationId)
                if (response.isSuccessful) {
                    onSuccess()
                } else {
                    onError("撤销邀请失败")
                }
            } catch (e: Exception) {
                onError(e.message ?: "撤销邀请失败")
            }
        }
    }

    /**
     * 通过邀请码加入房间
     */
    suspend fun joinByInvite(inviteCode: String): Boolean {
        return try {
            val request = JoinByInviteRequest(inviteCode)
            val response = roomApi.joinByInvite(request)
            response.isSuccessful && response.body()?.success == true
        } catch (e: Exception) {
            false
        }
    }

    /**
     * 验证邀请码
     */
    suspend fun validateInvite(inviteCode: String): Boolean {
        return try {
            val response = roomApi.validateInvite(inviteCode)
            response.isSuccessful && response.body()?.success == true
        } catch (e: Exception) {
            false
        }
    }

    /**
     * 复制邀请码到剪贴板
     */
    fun copyInviteCode(context: Context, code: String) {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clip = ClipData.newPlainText("邀请码", code)
        clipboard.setPrimaryClip(clip)
        Toast.makeText(context, "邀请码已复制", Toast.LENGTH_SHORT).show()
    }

    /**
     * 分享邀请链接
     */
    fun shareInvite(context: Context, code: String, roomName: String) {
        val shareText = "邀请你加入房间 \"$roomName\"\n邀请码: $code"
        val intent = Intent(Intent.ACTION_SEND).apply {
            type = "text/plain"
            putExtra(Intent.EXTRA_TEXT, shareText)
        }
        val chooser = Intent.createChooser(intent, "分享邀请")
        context.startActivity(chooser)
    }
}
