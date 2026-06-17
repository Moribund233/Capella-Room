use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;

/// 验证码用途
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationPurpose {
    Register,
    Login,
    ResetPassword,
}

impl VerificationPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationPurpose::Register => "register",
            VerificationPurpose::Login => "login",
            VerificationPurpose::ResetPassword => "reset_password",
        }
    }
}

impl FromStr for VerificationPurpose {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "register" => Ok(VerificationPurpose::Register),
            "login" => Ok(VerificationPurpose::Login),
            "reset_password" => Ok(VerificationPurpose::ResetPassword),
            _ => Err(()),
        }
    }
}

/// 验证码模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VerificationCode {
    pub id: Uuid,
    pub email: String,
    pub code: String,
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl VerificationCode {
    pub fn purpose_enum(&self) -> Option<VerificationPurpose> {
        self.purpose.parse().ok()
    }
}

/// 生成 6 位数字验证码
pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_returns_6_digits() {
        let code = generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_code_is_random() {
        let code1 = generate_code();
        let code2 = generate_code();
        assert_ne!(code1, code2);
    }

    #[test]
    fn test_verification_purpose_as_str() {
        assert_eq!(VerificationPurpose::Register.as_str(), "register");
        assert_eq!(VerificationPurpose::Login.as_str(), "login");
        assert_eq!(
            VerificationPurpose::ResetPassword.as_str(),
            "reset_password"
        );
    }

    #[test]
    fn test_verification_purpose_from_str() {
        assert_eq!(
            VerificationPurpose::from_str("register"),
            Ok(VerificationPurpose::Register)
        );
        assert_eq!(
            VerificationPurpose::from_str("login"),
            Ok(VerificationPurpose::Login)
        );
        assert_eq!(
            VerificationPurpose::from_str("reset_password"),
            Ok(VerificationPurpose::ResetPassword)
        );
        assert_eq!(VerificationPurpose::from_str("unknown"), Err(()));
    }

    #[test]
    fn test_verification_code_struct_fields() {
        let now = Utc::now();
        let code = VerificationCode {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            code: "123456".to_string(),
            purpose: "register".to_string(),
            expires_at: now,
            used_at: None,
            created_at: now,
        };
        assert_eq!(code.email, "test@example.com");
        assert_eq!(code.code, "123456");
        assert_eq!(code.purpose_enum(), Some(VerificationPurpose::Register));
        assert!(code.used_at.is_none());
    }
}
