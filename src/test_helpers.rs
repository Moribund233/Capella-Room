use std::sync::OnceLock;
use tokio::sync::Mutex;

/// 全局测试数据库互斥锁
/// 所有共享 `capella_room_test` 数据库的集成测试通过此锁串行化，
/// 避免并行执行时 `cleanup_database()` 跨测试删除数据。
pub fn db_guard() -> &'static Mutex<()> {
    static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    DB_LOCK.get_or_init(|| Mutex::new(()))
}
