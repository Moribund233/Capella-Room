package com.capella.room.data.local

import androidx.room.Database
import androidx.room.RoomDatabase
import com.capella.room.data.local.dao.MessageDao
import com.capella.room.data.local.dao.RoomDao
import com.capella.room.data.local.dao.UserDao
import com.capella.room.data.local.entity.CurrentUserEntity
import com.capella.room.data.local.entity.MessageEntity
import com.capella.room.data.local.entity.RoomEntity
import com.capella.room.data.local.entity.RoomMemberEntity
import com.capella.room.data.local.entity.UserEntity

/**
 * Capella Room 数据库
 * 包含消息、房间、用户等实体的本地存储
 */
@Database(
    entities = [
        MessageEntity::class,
        RoomEntity::class,
        RoomMemberEntity::class,
        UserEntity::class,
        CurrentUserEntity::class
    ],
    version = 1,
    exportSchema = true
)
abstract class CapellaDatabase : RoomDatabase() {

    abstract fun messageDao(): MessageDao
    abstract fun roomDao(): RoomDao
    abstract fun userDao(): UserDao

    companion object {
        const val DATABASE_NAME = "capella_room.db"
    }
}
