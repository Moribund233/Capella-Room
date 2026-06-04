package com.capella.room.di

import android.content.Context
import androidx.room.Room
import com.capella.room.data.local.CapellaDatabase
import com.capella.room.data.local.dao.MessageDao
import com.capella.room.data.local.dao.RoomDao
import com.capella.room.data.local.dao.UserDao
import com.capella.room.data.local.migration.DatabaseMigrations
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

/**
 * 数据库依赖注入模块
 */
@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {

    @Provides
    @Singleton
    fun provideDatabase(
        @ApplicationContext context: Context
    ): CapellaDatabase {
        return Room.databaseBuilder(
            context,
            CapellaDatabase::class.java,
            CapellaDatabase.DATABASE_NAME
        )
            // 生产环境使用迁移，开发阶段可以使用 fallbackToDestructiveMigration
            .addMigrations(*DatabaseMigrations.getAllMigrations())
            // .fallbackToDestructiveMigration() // 仅在开发阶段启用
            .build()
    }

    @Provides
    @Singleton
    fun provideMessageDao(database: CapellaDatabase): MessageDao {
        return database.messageDao()
    }

    @Provides
    @Singleton
    fun provideRoomDao(database: CapellaDatabase): RoomDao {
        return database.roomDao()
    }

    @Provides
    @Singleton
    fun provideUserDao(database: CapellaDatabase): UserDao {
        return database.userDao()
    }
}
