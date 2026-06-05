package com.capella.room.data.local.migration

import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

/**
 * 数据库迁移定义
 * 生产环境使用，开发阶段可以使用 fallbackToDestructiveMigration
 */
object DatabaseMigrations {

    /**
     * 从版本 1 迁移到版本 2
     * 添加消息已读回执字段
     */
    val MIGRATION_1_2 = object : Migration(1, 2) {
        override fun migrate(db: SupportSQLiteDatabase) {
            // 添加已读回执相关字段
            db.execSQL("ALTER TABLE messages ADD COLUMN is_read INTEGER NOT NULL DEFAULT 0")
            db.execSQL("ALTER TABLE messages ADD COLUMN read_count INTEGER NOT NULL DEFAULT 0")
            db.execSQL("ALTER TABLE messages ADD COLUMN read_by TEXT DEFAULT NULL")
            // 添加消息编辑相关字段
            db.execSQL("ALTER TABLE messages ADD COLUMN edited_at TEXT DEFAULT NULL")
        }
    }

    /**
     * 从版本 2 迁移到版本 3 的示例
     * 创建新表示例
     */
    val MIGRATION_2_3 = object : Migration(2, 3) {
        override fun migrate(db: SupportSQLiteDatabase) {
            // 示例：创建新表
            // db.execSQL("""
            //     CREATE TABLE IF NOT EXISTS message_reactions (
            //         message_id TEXT NOT NULL,
            //         user_id TEXT NOT NULL,
            //         reaction TEXT NOT NULL,
            //         created_at INTEGER NOT NULL,
            //         PRIMARY KEY(message_id, user_id, reaction)
            //     )
            // """)
        }
    }

    /**
     * 获取所有迁移
     */
    fun getAllMigrations(): Array<Migration> {
        return arrayOf(
            MIGRATION_1_2
        )
    }
}
