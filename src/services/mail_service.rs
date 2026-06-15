use crate::config::MailConfig;
use crate::error::Result;

#[derive(Clone)]
pub struct MailService {
    config: MailConfig,
}

impl MailService {
    pub fn new(config: MailConfig) -> Self {
        Self { config }
    }

    pub fn verification_code_ttl(&self) -> u64 {
        self.config.verification_code_ttl
    }

    pub fn from_address(&self) -> &str {
        &self.config.from_address
    }

    pub fn from_name(&self) -> &str {
        &self.config.from_name
    }

    pub async fn send_welcome(&self, username: &str, email: &str) -> Result<()> {
        tracing::info!(
            target: "mail",
            backend = "console",
            from = %self.config.from_address,
            email = %email,
            "[欢迎] {} -> {}: 欢迎 {username}", self.config.from_address, email
        );
        Ok(())
    }

    pub async fn send_password_reset(&self, email: &str, token: &str) -> Result<()> {
        tracing::info!(
            target: "mail",
            backend = "console",
            from = %self.config.from_address,
            email = %email,
            "[密码重置] {} -> {}: token={}", self.config.from_address, email, token
        );
        Ok(())
    }

    pub async fn send_verification_code(&self, email: &str, code: &str) -> Result<()> {
        tracing::info!(
            target: "mail",
            backend = "console",
            from = %self.config.from_address,
            email = %email,
            "[验证码] {} -> {}: {}", self.config.from_address, email, code
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MailBackend;

    fn create_test_config() -> MailConfig {
        MailConfig {
            backend: MailBackend::Console,
            from_address: "noreply@test.local".to_string(),
            from_name: "TestRoom".to_string(),
            verification_code_ttl: 10,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_send_verification_code_returns_ok() {
        let service = MailService::new(create_test_config());
        let result = service.send_verification_code("user@test.com", "123456").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_password_reset_returns_ok() {
        let service = MailService::new(create_test_config());
        let result = service.send_password_reset("user@test.com", "reset-token-abc").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_welcome_returns_ok() {
        let service = MailService::new(create_test_config());
        let result = service.send_welcome("Alice", "alice@test.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mail_service_send_to_empty_email() {
        let service = MailService::new(create_test_config());
        assert!(service.send_verification_code("", "123456").await.is_ok());
    }

    #[tokio::test]
    async fn test_mail_service_send_with_empty_code() {
        let service = MailService::new(create_test_config());
        assert!(service.send_verification_code("user@test.com", "").await.is_ok());
    }
}
