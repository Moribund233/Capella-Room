use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use tracing::{error, info};

use crate::config::{MailBackend, MailConfig};
use crate::error::Result;

const WELCOME_HTML: &str = include_str!("templates/welcome.html");
const PASSWORD_RESET_HTML: &str = include_str!("templates/password_reset.html");
const VERIFICATION_CODE_HTML: &str = include_str!("templates/verification_code.html");

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
        let subject = format!("欢迎加入 Capella Room，{}！", username);
        let html = WELCOME_HTML.replace("{{username}}", username);
        let plain = format!(
            "欢迎加入 Capella Room，{}！\n\n感谢您注册 Capella Room，现在您可以开始使用所有功能了。\n\n---\nCapella Room 实时聊天室平台",
            username
        );
        self.send(email, &subject, &html, &plain).await
    }

    pub async fn send_password_reset(&self, email: &str, token: &str) -> Result<()> {
        let subject = "Capella Room — 密码重置";
        let ttl = self.config.verification_code_ttl.to_string();
        let html = PASSWORD_RESET_HTML
            .replace("{{token}}", token)
            .replace("{{ttl}}", &ttl);
        let plain = format!(
            "密码重置\n\n您的密码重置验证码为：{token}\n有效期：{ttl} 分钟\n\n如果这不是您本人操作，请忽略此邮件。\n\n---\nCapella Room 实时聊天室平台",
            token = token,
            ttl = ttl
        );
        self.send(email, subject, &html, &plain).await
    }

    pub async fn send_verification_code(&self, email: &str, code: &str) -> Result<()> {
        let subject = "Capella Room — 邮箱验证";
        let ttl = self.config.verification_code_ttl.to_string();
        let html = VERIFICATION_CODE_HTML
            .replace("{{code}}", code)
            .replace("{{ttl}}", &ttl);
        let plain = format!(
            "邮箱验证\n\n您的验证码为：{code}\n有效期：{ttl} 分钟\n\n如果这不是您本人操作，请忽略此邮件。\n\n---\nCapella Room 实时聊天室平台",
            code = code,
            ttl = ttl
        );
        self.send(email, subject, &html, &plain).await
    }

    async fn send(&self, to: &str, subject: &str, html_body: &str, plain_body: &str) -> Result<()> {
        match self.config.backend {
            MailBackend::Console => {
                info!(
                    target: "mail",
                    backend = "console",
                    from = %self.config.from_address,
                    to = %to,
                    subject = %subject,
                    "[邮件] {} -> {}: {}", self.config.from_address, to, subject
                );
                Ok(())
            }
            MailBackend::Smtp => {
                self.send_smtp(to, subject, html_body, plain_body).await
            }
        }
    }

    async fn send_smtp(&self, to: &str, subject: &str, html_body: &str, plain_body: &str) -> Result<()> {
        let from: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_address)
            .parse()
            .map_err(|e| {
                error!("无效的发件人地址: {} - {}", self.config.from_address, e);
                crate::error::AppError::Config("无效的邮件发件人配置".to_string())
            })?;

        let to: Mailbox = to.parse().map_err(|e| {
            error!("无效的收件人地址: {} - {}", to, e);
            crate::error::AppError::Validation(format!("无效的邮箱地址: {}", to))
        })?;

        let message = Message::builder()
            .from(from)
            .to(to.clone())
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(lettre::message::SinglePart::plain(plain_body.to_string()))
                    .singlepart(lettre::message::SinglePart::html(html_body.to_string())),
            )
            .map_err(|e| {
                error!("构建邮件失败: {}", e);
                crate::error::AppError::Internal
            })?;

        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        let mailer = if self.config.smtp_use_tls && self.config.smtp_port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
                .map_err(|e| {
                    error!("SMTP 中继配置失败: {}", e);
                    crate::error::AppError::Config("SMTP 配置错误".to_string())
                })?
                .port(self.config.smtp_port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.smtp_host)
                .map_err(|e| {
                    error!("SMTP STARTTLS 配置失败: {}", e);
                    crate::error::AppError::Config("SMTP 配置错误".to_string())
                })?
                .port(self.config.smtp_port)
                .credentials(creds)
                .build()
        };

        match mailer.send(message).await {
            Ok(_) => {
                info!(
                    target: "mail",
                    backend = "smtp",
                    from = %self.config.from_address,
                    to = %to,
                    subject = %subject,
                    "[邮件][SMTP] {} -> {}: {} 发送成功",
                    self.config.from_address, to, subject
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    target: "mail",
                    backend = "smtp",
                    from = %self.config.from_address,
                    to = %to,
                    error = %e,
                    "[邮件][SMTP] 发送失败: {} -> {}: {} - {}",
                    self.config.from_address, to, subject, e
                );
                Err(crate::error::AppError::WebSocket(format!("邮件发送失败: {}", e)))
            }
        }
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

    #[tokio::test]
    async fn test_smtp_backend_rejects_invalid_from() {
        let config = MailConfig {
            backend: MailBackend::Smtp,
            from_address: "".to_string(),
            from_name: "".to_string(),
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "test".to_string(),
            smtp_password: "test".to_string(),
            smtp_use_tls: true,
            verification_code_ttl: 10,
        };
        let service = MailService::new(config);
        let result = service.send_verification_code("user@test.com", "123456").await;
        assert!(result.is_err());
    }
}
