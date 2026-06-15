use chrono::{DateTime, Utc};

use crate::{
    db::Database,
    error::Result,
    models::verification_code::{generate_code, VerificationCode, VerificationPurpose},
    services::mail_service::MailService,
};

#[derive(Clone)]
pub struct VerificationCodeService {
    db: Database,
    mail_service: MailService,
}

impl VerificationCodeService {
    pub fn new(db: Database, mail_service: MailService) -> Self {
        Self { db, mail_service }
    }

    pub async fn create_code(
        &self,
        email: &str,
        purpose: VerificationPurpose,
    ) -> Result<VerificationCode> {
        if !can_resend_internal(self.db.pool(), email, &purpose).await? {
            return Err(crate::error::AppError::Validation(
                "验证码发送过于频繁，请稍后再试".to_string(),
            ));
        }

        let code = generate_code();
        let ttl_minutes = self.mail_service.verification_code_ttl() as i64;
        let expires_at = Utc::now() + chrono::Duration::minutes(ttl_minutes);

        let record = sqlx::query_as::<_, VerificationCode>(
            r#"
            INSERT INTO verification_codes (email, code, purpose, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, code, purpose, expires_at, used_at, created_at
            "#,
        )
        .bind(email)
        .bind(&code)
        .bind(purpose.as_str())
        .bind(expires_at)
        .fetch_one(self.db.pool())
        .await?;

        self.mail_service
            .send_verification_code(email, &code)
            .await?;

        Ok(record)
    }

    pub async fn verify_code(
        &self,
        email: &str,
        code: &str,
        purpose: VerificationPurpose,
    ) -> Result<bool> {
        let record = sqlx::query_as::<_, VerificationCode>(
            r#"
            SELECT id, email, code, purpose, expires_at, used_at, created_at
            FROM verification_codes
            WHERE email = $1 AND purpose = $2 AND used_at IS NULL
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email)
        .bind(purpose.as_str())
        .fetch_optional(self.db.pool())
        .await?;

        match record {
            Some(vc) => {
                if is_code_expired(vc.expires_at) {
                    return Ok(false);
                }
                if !matches_code(code, &vc.code) {
                    return Ok(false);
                }
                sqlx::query("UPDATE verification_codes SET used_at = NOW() WHERE id = $1")
                    .bind(vc.id)
                    .execute(self.db.pool())
                    .await?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn can_resend(&self, email: &str, purpose: VerificationPurpose) -> Result<bool> {
        can_resend_internal(self.db.pool(), email, &purpose).await
    }
}

async fn can_resend_internal(
    pool: &sqlx::PgPool,
    email: &str,
    purpose: &VerificationPurpose,
) -> Result<bool> {
    let cooldown_secs: i64 = 60;

    let last: Option<(DateTime<Utc>,)> = sqlx::query_as(
        r#"
        SELECT created_at FROM verification_codes
        WHERE email = $1 AND purpose = $2
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(email)
    .bind(purpose.as_str())
    .fetch_optional(pool)
    .await?;

    match last {
        Some((created_at,)) => Ok(has_cooldown_passed(created_at, Utc::now(), cooldown_secs)),
        None => Ok(true),
    }
}

fn is_code_expired(expires_at: DateTime<Utc>) -> bool {
    Utc::now() > expires_at
}

fn matches_code(provided: &str, stored: &str) -> bool {
    provided == stored
}

fn has_cooldown_passed(
    last_sent_at: DateTime<Utc>,
    now: DateTime<Utc>,
    cooldown_secs: i64,
) -> bool {
    now >= last_sent_at + chrono::Duration::seconds(cooldown_secs)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;

    #[test]
    fn test_service_can_be_created() {
        // Type exists, constructor compiles
    }

    #[test]
    fn test_is_code_expired() {
        let future = Utc::now() + Duration::minutes(5);
        assert!(!is_code_expired(future));

        let past = Utc::now() - Duration::minutes(5);
        assert!(is_code_expired(past));
    }

    #[test]
    fn test_matches_code() {
        assert!(matches_code("123456", "123456"));
        assert!(!matches_code("123456", "654321"));
        assert!(!matches_code("123456", ""));
    }

    #[test]
    fn test_has_cooldown_passed() {
        let now = Utc::now();
        let cooldown_secs: i64 = 60;

        let recent = now - Duration::seconds(30);
        assert!(!has_cooldown_passed(recent, now, cooldown_secs));

        let old_enough = now - Duration::seconds(90);
        assert!(has_cooldown_passed(old_enough, now, cooldown_secs));

        let exactly_at_limit = now - Duration::seconds(60);
        assert!(has_cooldown_passed(exactly_at_limit, now, cooldown_secs));
    }
}
