use std::time::Duration;

/// 重连策略配置
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 初始退避时间（毫秒）
    pub base_delay_ms: u64,
    /// 最大退避时间（毫秒）
    pub max_delay_ms: u64,
    /// 最大重连尝试次数
    pub max_attempts: u32,
    /// 退避乘数（指数因子）
    pub multiplier: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 1000, // 1 秒
            max_delay_ms: 30000, // 30 秒
            max_attempts: 5,     // 最多 5 次尝试
            multiplier: 2,       // 指数退避因子
        }
    }
}

/// 重连策略
/// 实现指数退避算法
pub struct ReconnectStrategy {
    config: ReconnectConfig,
    attempt: u32,
}

impl ReconnectStrategy {
    /// 创建新的重连策略
    pub fn new(config: ReconnectConfig) -> Self {
        Self { config, attempt: 0 }
    }

    /// 创建默认配置的重连策略
    pub fn with_default_config() -> Self {
        Self::new(ReconnectConfig::default())
    }

    /// 获取下一次重连的延迟时间
    /// 返回 None 表示已达到最大尝试次数
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.attempt >= self.config.max_attempts {
            return None;
        }

        // 计算指数退避延迟
        let delay = self.config.base_delay_ms
            * (self.config.multiplier as u64).saturating_pow(self.attempt);

        // 限制在最大延迟范围内
        let delay = delay.min(self.config.max_delay_ms);

        self.attempt += 1;

        Some(Duration::from_millis(delay))
    }

    /// 重置策略（用于成功重连后）
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// 获取当前尝试次数
    pub fn current_attempt(&self) -> u32 {
        self.attempt
    }

    /// 是否还可以继续尝试
    pub fn can_retry(&self) -> bool {
        self.attempt < self.config.max_attempts
    }

    /// 计算指定尝试次数的延迟（不改变内部状态）
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay =
            self.config.base_delay_ms * (self.config.multiplier as u64).saturating_pow(attempt);
        Duration::from_millis(delay.min(self.config.max_delay_ms))
    }
}

/// 计算指数退避延迟（工具函数）
pub fn exponential_backoff(
    attempt: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: u32,
) -> Duration {
    let delay = base_delay_ms * (multiplier as u64).saturating_pow(attempt);
    Duration::from_millis(delay.min(max_delay_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_sequence() {
        let mut strategy = ReconnectStrategy::with_default_config();

        // 尝试 1: 1 秒
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(1000)));
        // 尝试 2: 2 秒
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(2000)));
        // 尝试 3: 4 秒
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(4000)));
        // 尝试 4: 8 秒
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(8000)));
        // 尝试 5: 16 秒
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(16000)));
        // 尝试 6: 无（达到最大尝试次数）
        assert_eq!(strategy.next_delay(), None);
    }

    #[test]
    fn test_max_delay_cap() {
        let config = ReconnectConfig {
            base_delay_ms: 1000,
            max_delay_ms: 5000, // 5 秒上限
            max_attempts: 10,
            multiplier: 2,
        };
        let mut strategy = ReconnectStrategy::new(config);

        // 延迟应该被限制在 5 秒
        for _ in 0..3 {
            let delay = strategy.next_delay().unwrap();
            assert!(delay <= Duration::from_millis(5000));
        }
    }

    #[test]
    fn test_reset() {
        let mut strategy = ReconnectStrategy::with_default_config();

        strategy.next_delay();
        strategy.next_delay();
        assert_eq!(strategy.current_attempt(), 2);

        strategy.reset();
        assert_eq!(strategy.current_attempt(), 0);
        assert!(strategy.can_retry());
    }

    #[test]
    fn test_delay_calculation() {
        let strategy = ReconnectStrategy::with_default_config();

        assert_eq!(strategy.delay_for_attempt(0), Duration::from_millis(1000));
        assert_eq!(strategy.delay_for_attempt(1), Duration::from_millis(2000));
        assert_eq!(strategy.delay_for_attempt(2), Duration::from_millis(4000));
        assert_eq!(strategy.delay_for_attempt(3), Duration::from_millis(8000));
    }

    #[test]
    fn test_custom_config() {
        let config = ReconnectConfig {
            base_delay_ms: 500,
            max_delay_ms: 10000,
            max_attempts: 3,
            multiplier: 3,
        };
        let mut strategy = ReconnectStrategy::new(config);

        // 尝试 1: 500ms
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(500)));
        // 尝试 2: 1500ms (500 * 3)
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(1500)));
        // 尝试 3: 4500ms (500 * 3^2)
        assert_eq!(strategy.next_delay(), Some(Duration::from_millis(4500)));
        // 尝试 4: 无
        assert_eq!(strategy.next_delay(), None);
    }
}
