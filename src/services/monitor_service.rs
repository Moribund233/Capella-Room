use serde::Serialize;
use sysinfo::{Disks, System};

use crate::db::Database;
use crate::error::Result;

/// 系统监控信息
#[derive(Debug, Clone, Serialize)]
pub struct SystemMonitorInfo {
    /// 内存使用情况
    pub memory: MemoryInfo,
    /// 磁盘使用情况
    pub disk: DiskInfo,
    /// 应用进程内存占用 (MB)
    pub process_memory_mb: u64,
}

/// 内存信息
#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    /// 总内存 (MB)
    pub total_mb: u64,
    /// 已使用内存 (MB)
    pub used_mb: u64,
    /// 可用内存 (MB)
    pub available_mb: u64,
    /// 使用率 (%)
    pub usage_percent: f64,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    /// 总空间 (GB)
    pub total_gb: u64,
    /// 已使用空间 (GB)
    pub used_gb: u64,
    /// 可用空间 (GB)
    pub available_gb: u64,
    /// 使用率 (%)
    pub usage_percent: f64,
}

/// 数据库连接池信息
#[derive(Debug, Clone, Serialize)]
pub struct DatabasePoolInfo {
    /// 最大连接数
    pub max_connections: u32,
    /// 当前活跃连接数
    pub active_connections: u32,
    /// 空闲连接数
    pub idle_connections: u32,
    /// 等待连接的请求数
    pub waiting_requests: u32,
}

/// 综合监控数据
#[derive(Debug, Clone, Serialize)]
pub struct MonitorData {
    /// 系统监控信息
    pub system: SystemMonitorInfo,
    /// 数据库连接池信息
    pub database: DatabasePoolInfo,
    /// 时间戳
    pub timestamp: String,
}

/// 监控服务
pub struct MonitorService {
    db: Database,
}

impl MonitorService {
    /// 创建新的监控服务
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 获取系统监控信息
    pub fn get_system_info() -> SystemMonitorInfo {
        let mut system = System::new_all();
        system.refresh_all();

        // 获取内存信息
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();

        let memory = MemoryInfo {
            total_mb: total_memory / 1024,
            used_mb: used_memory / 1024,
            available_mb: available_memory / 1024,
            usage_percent: if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            },
        };

        // 获取磁盘信息
        let disks = Disks::new_with_refreshed_list();
        let mut total_space = 0;
        let mut available_space = 0;

        for disk in &disks {
            total_space += disk.total_space();
            available_space += disk.available_space();
        }

        let used_space = total_space.saturating_sub(available_space);

        let disk = DiskInfo {
            total_gb: total_space / 1024 / 1024 / 1024,
            used_gb: used_space / 1024 / 1024 / 1024,
            available_gb: available_space / 1024 / 1024 / 1024,
            usage_percent: if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            },
        };

        // 获取当前进程内存占用
        let current_pid = sysinfo::Pid::from_u32(std::process::id());
        let process_memory_mb = system
            .process(current_pid)
            .map(|p| p.memory() / 1024)
            .unwrap_or(0);

        SystemMonitorInfo {
            memory,
            disk,
            process_memory_mb,
        }
    }

    /// 获取数据库连接池信息
    pub async fn get_database_pool_info(&self) -> Result<DatabasePoolInfo> {
        let pool = self.db.pool();

        // sqlx 的连接池信息获取有限，这里返回配置值和估算值
        // 实际活跃连接数需要通过数据库查询获取
        let active_connections = self.get_active_db_connections().await?;

        Ok(DatabasePoolInfo {
            max_connections: pool.options().get_max_connections(),
            active_connections: active_connections as u32,
            idle_connections: pool.options().get_max_connections() - active_connections as u32,
            waiting_requests: 0, // sqlx 不直接暴露这个信息
        })
    }

    /// 获取数据库活跃连接数
    async fn get_active_db_connections(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database()",
        )
        .fetch_one(self.db.pool())
        .await?;

        Ok(row.0)
    }

    /// 获取综合监控数据
    pub async fn get_monitor_data(&self) -> Result<MonitorData> {
        let system = Self::get_system_info();
        let database = self.get_database_pool_info().await?;

        Ok(MonitorData {
            system,
            database,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_info() {
        let info = MonitorService::get_system_info();
        assert!(info.memory.total_mb > 0);
        assert!(info.disk.total_gb > 0);
    }
}
