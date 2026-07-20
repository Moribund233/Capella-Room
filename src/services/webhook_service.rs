use chrono::{Duration, Utc};
use std::net::IpAddr;
use tracing::warn;
use uuid::Uuid;

use crate::{
    db::Database,
    error::{AppError, Result},
    models::webhook::{WebhookDelivery, WebhookSubscription},
};

pub struct WebhookService {
    db: Database,
    http_client: reqwest::Client,
}

impl WebhookService {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            http_client: reqwest::Client::new(),
        }
    }

    // ─── Subscription CRUD ───

    pub async fn create_subscription(
        &self,
        app_id: Uuid,
        url: &str,
        secret: &str,
        events: &[String],
    ) -> Result<WebhookSubscription> {
        if is_private_url(url) {
            return Err(AppError::Validation(
                "Webhook URL 不能指向内网地址".to_string(),
            ));
        }

        let sub = sqlx::query_as::<_, WebhookSubscription>(
            r#"INSERT INTO webhook_subscriptions (app_id, url, secret, events)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#
        )
        .bind(app_id)
        .bind(url)
        .bind(secret)
        .bind(events)
        .fetch_one(self.db.pool())
        .await?;
        Ok(sub)
    }

    pub async fn list_subscriptions(&self, app_id: Uuid) -> Result<Vec<WebhookSubscription>> {
        let subs = sqlx::query_as::<_, WebhookSubscription>(
            "SELECT * FROM webhook_subscriptions WHERE app_id = $1 ORDER BY created_at DESC"
        )
        .bind(app_id)
        .fetch_all(self.db.pool())
        .await?;
        Ok(subs)
    }

    pub async fn get_subscription(&self, sub_id: Uuid, app_id: Uuid) -> Result<WebhookSubscription> {
        let sub = sqlx::query_as::<_, WebhookSubscription>(
            "SELECT * FROM webhook_subscriptions WHERE id = $1 AND app_id = $2"
        )
        .bind(sub_id)
        .bind(app_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AppError::NotFound)?;
        Ok(sub)
    }

    pub async fn update_subscription(
        &self,
        sub_id: Uuid,
        app_id: Uuid,
        url: Option<&str>,
        secret: Option<&str>,
        events: Option<&[String]>,
        is_active: Option<bool>,
    ) -> Result<WebhookSubscription> {
        let existing = self.get_subscription(sub_id, app_id).await?;

        let new_url = url.unwrap_or(&existing.url);
        if url.is_some() && is_private_url(new_url) {
            return Err(AppError::Validation(
                "Webhook URL 不能指向内网地址".to_string(),
            ));
        }

        let new_secret = secret.unwrap_or(&existing.secret);
        let new_events = events.unwrap_or(&existing.events);
        let new_active = is_active.unwrap_or(existing.is_active);

        let sub = sqlx::query_as::<_, WebhookSubscription>(
            r#"UPDATE webhook_subscriptions
               SET url = $1, secret = $2, events = $3, is_active = $4, updated_at = NOW()
               WHERE id = $5 AND app_id = $6
               RETURNING *"#
        )
        .bind(new_url)
        .bind(new_secret)
        .bind(new_events)
        .bind(new_active)
        .bind(sub_id)
        .bind(app_id)
        .fetch_one(self.db.pool())
        .await?;
        Ok(sub)
    }

    pub async fn delete_subscription(&self, sub_id: Uuid, app_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM webhook_subscriptions WHERE id = $1 AND app_id = $2"
        )
        .bind(sub_id)
        .bind(app_id)
        .execute(self.db.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    // ─── Event Dispatch ───

    pub async fn dispatch_event(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<usize> {
        let subs = sqlx::query_as::<_, WebhookSubscription>(
            r#"SELECT * FROM webhook_subscriptions
               WHERE is_active = true AND $1 = ANY(events)"#
        )
        .bind(event_type)
        .fetch_all(self.db.pool())
        .await?;

        let count = subs.len();
        if count == 0 {
            return Ok(0);
        }

        let event_id = Uuid::new_v4();
        let event_type_owned = event_type.to_string();
        for sub in subs {
            let delivery_id = Uuid::new_v4();
            let payload_clone = payload.clone();
            sqlx::query(
                r#"INSERT INTO webhook_deliveries
                   (id, subscription_id, event_type, event_id, payload, status, attempt_count, next_retry_at)
                   VALUES ($1, $2, $3, $4, $5, 'pending', 1, NOW())"#
            )
            .bind(delivery_id)
            .bind(sub.id)
            .bind(&event_type_owned)
            .bind(event_id)
            .bind(&payload_clone)
            .execute(self.db.pool())
            .await?;

            let http_client = self.http_client.clone();
            let db = self.db.clone();
            let sub_url = sub.url.clone();
            let sub_secret = sub.secret.clone();
            let sub_timeout = sub.timeout_ms;
            let sub_retry_interval = sub.retry_interval_secs;
            let evt = event_type_owned.clone();
            tokio::spawn(async move {
                let result = deliver_once(
                    &http_client, &sub_url, &sub_secret,
                    &evt, event_id, &payload_clone, sub_timeout, 1,
                ).await;

                let (status, http_status, response_body) = match result {
                    Ok((code, body)) => ("success".to_string(), Some(code), Some(body)),
                    Err(e) => {
                        warn!("webhook delivery failed for {}: {}", sub_url, e);
                        ("failed".to_string(), None, Some(e.to_string()))
                    }
                };

                let _ = sqlx::query(
                    r#"UPDATE webhook_deliveries
                       SET status = $1, http_status = $2, response_body = $3,
                           completed_at = CASE WHEN $1 = 'success' THEN NOW() ELSE completed_at END,
                           next_retry_at = CASE WHEN $1 = 'failed' THEN NOW() + $5 * INTERVAL '1 second' ELSE NULL END
                       WHERE id = $4"#
                )
                .bind(&status)
                .bind(http_status)
                .bind(response_body)
                .bind(delivery_id)
                .bind(sub_retry_interval)
                .execute(db.pool())
                .await;
            });
        }

        Ok(count)
    }

    // ─── Delivery Records ───

    pub async fn get_deliveries(
        &self,
        sub_id: Uuid,
        app_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDelivery>> {
        // Verify ownership
        let _ = self.get_subscription(sub_id, app_id).await?;

        let deliveries = sqlx::query_as::<_, WebhookDelivery>(
            r#"SELECT * FROM webhook_deliveries
               WHERE subscription_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#
        )
        .bind(sub_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db.pool())
        .await?;
        Ok(deliveries)
    }

    pub async fn redeliver(
        &self,
        delivery_id: Uuid,
        sub_id: Uuid,
        app_id: Uuid,
    ) -> Result<()> {
        let sub = self.get_subscription(sub_id, app_id).await?;

        let delivery = sqlx::query_as::<_, WebhookDelivery>(
            "SELECT * FROM webhook_deliveries WHERE id = $1 AND subscription_id = $2"
        )
        .bind(delivery_id)
        .bind(sub_id)
        .fetch_optional(self.db.pool())
        .await?
        .ok_or(AppError::NotFound)?;

        // Reset delivery for retry
        sqlx::query(
            r#"UPDATE webhook_deliveries
               SET status = 'pending', attempt_count = attempt_count + 1,
                   next_retry_at = NOW(), http_status = NULL, response_body = NULL
               WHERE id = $1"#
        )
        .bind(delivery_id)
        .execute(self.db.pool())
        .await?;

        // Spawn immediate retry
        let http_client = self.http_client.clone();
        let db = self.db.clone();
        let sub_url = sub.url.clone();
        let sub_secret = sub.secret.clone();
        let sub_timeout = sub.timeout_ms;
        let attempt = delivery.attempt_count + 1;

        tokio::spawn(async move {
            let result = deliver_once(
                &http_client, &sub_url, &sub_secret,
                &delivery.event_type, delivery.event_id, &delivery.payload,
                sub_timeout, attempt,
            ).await;

            let (status, http_status, response_body) = match result {
                Ok((code, body)) => ("success".to_string(), Some(code), Some(body)),
                Err(e) => ("failed".to_string(), None, Some(e.to_string())),
            };

            let _ = sqlx::query(
                r#"UPDATE webhook_deliveries
                   SET status = $1, http_status = $2, response_body = $3,
                       completed_at = CASE WHEN $1 = 'success' THEN NOW() ELSE completed_at END
                   WHERE id = $4"#
            )
            .bind(&status)
            .bind(http_status)
            .bind(response_body)
            .bind(delivery_id)
                .execute(db.pool())
            .await;
        });

        Ok(())
    }

    // ─── Background Retry ───

    pub async fn retry_failed_deliveries(&self) -> Result<usize> {
        let pending = sqlx::query_as::<_, WebhookDelivery>(
            r#"SELECT d.* FROM webhook_deliveries d
               JOIN webhook_subscriptions s ON d.subscription_id = s.id
               WHERE d.status IN ('pending', 'failed')
                 AND d.next_retry_at <= NOW()
                 AND s.is_active = true
               ORDER BY d.next_retry_at
               LIMIT 50"#
        )
        .fetch_all(self.db.pool())
        .await?;

        let count = pending.len();
        for delivery in pending {
            let sub = sqlx::query_as::<_, WebhookSubscription>(
                "SELECT * FROM webhook_subscriptions WHERE id = $1"
            )
            .bind(delivery.subscription_id)
            .fetch_optional(self.db.pool())
            .await?;

            let sub = match sub {
                Some(s) => s,
                None => continue,
            };

            let http_client = self.http_client.clone();
            let db = self.db.clone();
            let sub_url = sub.url.clone();
            let sub_secret = sub.secret.clone();
            let sub_timeout = sub.timeout_ms;
            let attempt = delivery.attempt_count + 1;

            tokio::spawn(async move {
                let result = deliver_once(
                    &http_client, &sub_url, &sub_secret,
                    &delivery.event_type, delivery.event_id, &delivery.payload,
                    sub_timeout, attempt,
                ).await;

                let (status, http_status, response_body) = match result {
                    Ok((code, body)) => ("success".to_string(), Some(code), Some(body)),
                    Err(e) => ("failed".to_string(), None, Some(e.to_string())),
                };

                let next_retry = if status == "failed" && attempt < sub.max_retries {
                    let backoff_secs = sub.retry_interval_secs as i64 * 2i64.pow((attempt - 1) as u32);
                    Some(Utc::now() + Duration::seconds(backoff_secs))
                } else {
                    None
                };

                let _ = sqlx::query(
                    r#"UPDATE webhook_deliveries
                       SET status = $1, http_status = $2, response_body = $3,
                           attempt_count = $4,
                           completed_at = CASE WHEN $1 = 'success' THEN NOW() ELSE completed_at END,
                           next_retry_at = $5
                       WHERE id = $6"#
                )
                .bind(&status)
                .bind(http_status)
                .bind(response_body)
                .bind(attempt)
                .bind(next_retry)
                .bind(delivery.id)
                .execute(db.pool())
                .await;
            });
        }

        Ok(count)
    }
}

// ─── Standalone delivery function ───

/// 检查 IP 地址是否属于私有/内网地址
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_unspecified()
        }
        IpAddr::V6(ip) => {
            // IPv6 回环与未指定地址；独特本地地址（fc00::/7）和链路本地（fe80::/10）
            // 通过地址段前缀判断
            ip.is_loopback()
                || ip.is_unspecified()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

/// 检查 Webhook URL 是否指向内网或私有地址，防止 SSRF
fn is_private_url(url_str: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url_str) else {
        return true; // 无效 URL 视为不安全
    };

    if let Some(host) = parsed.host_str() {
        // 解析为 IP 地址后检查是否为私有/内网地址
        if let Ok(ip) = host.parse::<IpAddr>() {
            return is_private_ip(ip);
        }

        // 域名检查：禁止常见内网域名或以内网网段开头的域名
        let private_domains = [
            "localhost", "127.0.0.1", "0.0.0.0", "[::1]", "[::]", "10.", "172.16.",
            "192.168.", "169.254.",
        ];
        if private_domains.iter().any(|d| host.starts_with(d)) {
            return true;
        }
    }

    false
}

#[allow(clippy::too_many_arguments)]
async fn deliver_once(
    http_client: &reqwest::Client,
    url: &str,
    secret: &str,
    event_type: &str,
    _event_id: Uuid,
    payload: &serde_json::Value,
    timeout_ms: i32,
    attempt: i32,
) -> Result<(i32, String)> {
    if is_private_url(url) {
        return Err(AppError::Validation(
            "Webhook URL 不能指向内网地址".to_string(),
        ));
    }

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let body = serde_json::to_string(payload).map_err(|_e| AppError::Internal)?;
    let timestamp = Utc::now().timestamp();

    // HMAC-SHA256 signing
    let msg = format!("{}.{}", timestamp, &body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::Internal)?;
    mac.update(msg.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let delivery_id = Uuid::new_v4();
    let timeout = std::time::Duration::from_millis(timeout_ms as u64);

    let response = http_client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Capella-Signature", format!("sha256={}", signature))
        .header("X-Capella-Timestamp", timestamp.to_string())
        .header("X-Capella-Event-Type", event_type)
        .header("X-Capella-Delivery-Id", delivery_id.to_string())
        .header("X-Capella-Attempt", attempt.to_string())
        .timeout(timeout)
        .json(payload)
        .send()
        .await
        .map_err(|e| AppError::Auth(format!("webhook request failed: {}", e)))?;

    let status = response.status().as_u16() as i32;
    let body_text = response.text().await.unwrap_or_default();

    if (200..300).contains(&status) {
        Ok((status, body_text))
    } else {
        Err(AppError::Auth(format!("webhook returned {}: {}", status, body_text)))
    }
}

impl Clone for WebhookService {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_private_url;

    #[test]
    fn test_private_url_blocks_localhost() {
        assert!(is_private_url("http://localhost:8080/webhook"));
        assert!(is_private_url("http://127.0.0.1:8080/webhook"));
    }

    #[test]
    fn test_private_url_blocks_private_ips() {
        assert!(is_private_url("http://192.168.1.1/webhook"));
        assert!(is_private_url("http://10.0.0.1/webhook"));
        assert!(is_private_url("http://172.16.0.1/webhook"));
    }

    #[test]
    fn test_private_url_blocks_invalid_url() {
        assert!(is_private_url("not-a-valid-url"));
    }

    #[test]
    fn test_private_url_allows_public_urls() {
        assert!(!is_private_url("https://example.com/webhook"));
        assert!(!is_private_url("https://hooks.example.com/path"));
    }
}
