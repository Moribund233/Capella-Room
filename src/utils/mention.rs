use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::db::Database;

/// 正则表达式：匹配 @username 格式
/// 用户名规则：字母、数字、下划线，长度3-20
/// 使用单词边界确保匹配完整的用户名
static MENTION_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"@(\w{3,20})\b").unwrap());

/// 从消息内容中提取@提及的用户名
///
/// # 参数
/// - `content`: 消息内容
///
/// # 返回值
/// 返回被提及的用户名列表（不包含@符号）
///
/// # 示例
/// ```
/// let content = "Hello @alice and @bob!";
/// let mentions = extract_mentions(content);
/// assert_eq!(mentions, vec!["alice", "bob"]);
/// ```
pub fn extract_mentions(content: &str) -> Vec<String> {
    let mentions: Vec<String> = MENTION_PATTERN
        .captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_lowercase()))
        .collect();

    // 去重
    let mut unique_mentions = mentions;
    unique_mentions.sort();
    unique_mentions.dedup();

    debug!("Extracted {} mentions from message", unique_mentions.len());
    unique_mentions
}

/// 将用户名列表解析为用户ID列表
///
/// # 参数
/// - `usernames`: 用户名列表
/// - `db`: 数据库连接
///
/// # 返回值
/// 返回成功解析的用户ID列表
///
/// # 说明
/// - 不存在的用户名会被忽略
/// - 用户名匹配不区分大小写
pub async fn resolve_usernames_to_ids(
    usernames: Vec<String>,
    db: &Database,
) -> anyhow::Result<Vec<Uuid>> {
    if usernames.is_empty() {
        return Ok(Vec::new());
    }

    let mut user_ids = Vec::new();

    for username in usernames {
        match sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT id FROM users WHERE LOWER(username) = LOWER($1)
            "#,
        )
        .bind(&username)
        .fetch_optional(db.pool())
        .await
        {
            Ok(Some((user_id,))) => {
                user_ids.push(user_id);
                debug!("Resolved username '{}' to user ID: {}", username, user_id);
            }
            Ok(None) => {
                debug!("Username '{}' not found, skipping", username);
            }
            Err(e) => {
                warn!("Failed to resolve username '{}': {}", username, e);
            }
        }
    }

    Ok(user_ids)
}

/// 检查消息中是否包含@提及
///
/// # 参数
/// - `content`: 消息内容
///
/// # 返回值
/// 如果包含@提及返回true，否则返回false
pub fn has_mentions(content: &str) -> bool {
    !extract_mentions(content).is_empty()
}

/// 获取消息中@提及的数量
///
/// # 参数
/// - `content`: 消息内容
///
/// # 返回值
/// 返回@提及的唯一用户数量
pub fn mention_count(content: &str) -> usize {
    extract_mentions(content).len()
}

/// 提取@提及并解析为用户ID（便捷方法）
///
/// # 参数
/// - `content`: 消息内容
/// - `db`: 数据库连接
///
/// # 返回值
/// 返回被提及用户的ID列表
pub async fn extract_and_resolve_mentions(
    content: &str,
    db: &Database,
) -> anyhow::Result<Vec<Uuid>> {
    let usernames = extract_mentions(content);
    resolve_usernames_to_ids(usernames, db).await
}

/// 过滤掉发送者自己的用户名
///
/// # 参数
/// - `mentions`: 提及的用户名列表
/// - `sender_username`: 发送者的用户名
///
/// # 返回值
/// 返回过滤后的用户名列表（不包含发送者自己）
pub fn filter_self_mentions(mentions: Vec<String>, sender_username: &str) -> Vec<String> {
    mentions
        .into_iter()
        .filter(|name| name.to_lowercase() != sender_username.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mentions_single() {
        let content = "Hello @alice!";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["alice"]);
    }

    #[test]
    fn test_extract_mentions_multiple() {
        let content = "Hello @alice and @bob!";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["alice", "bob"]);
    }

    #[test]
    fn test_extract_mentions_duplicate() {
        let content = "@alice @alice @bob";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["alice", "bob"]);
    }

    #[test]
    fn test_extract_mentions_case_insensitive() {
        let content = "Hello @Alice and @BOB!";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["alice", "bob"]);
    }

    #[test]
    fn test_extract_mentions_no_mentions() {
        let content = "Hello everyone!";
        let mentions = extract_mentions(content);
        assert!(mentions.is_empty());
    }

    #[test]
    fn test_extract_mentions_invalid_format() {
        // 太短的用户名（少于3个字符）
        let content = "Hello @ab!";
        let mentions = extract_mentions(content);
        assert!(mentions.is_empty());

        // 太长的用户名（超过20个字符）
        let content = "Hello @verylongusernamethatistoolong!";
        let mentions = extract_mentions(content);
        assert!(mentions.is_empty());
    }

    #[test]
    fn test_extract_mentions_with_special_chars() {
        // 只包含字母、数字、下划线
        // @test-user 中的 test 符合长度要求（4个字符），会被匹配
        // @user_123 也会被匹配
        let content = "Hello @user_123 and @test-user!";
        let mentions = extract_mentions(content);
        assert_eq!(mentions, vec!["test", "user_123"]);
    }

    #[test]
    fn test_has_mentions() {
        assert!(has_mentions("Hello @alice!"));
        assert!(!has_mentions("Hello everyone!"));
    }

    #[test]
    fn test_mention_count() {
        assert_eq!(mention_count("Hello @alice and @bob!"), 2);
        assert_eq!(mention_count("Hello everyone!"), 0);
        assert_eq!(mention_count("@alice @alice @bob"), 2); // 去重后
    }

    #[test]
    fn test_filter_self_mentions() {
        let mentions = vec![
            "alice".to_string(),
            "bob".to_string(),
            "charlie".to_string(),
        ];
        let filtered = filter_self_mentions(mentions, "alice");
        assert_eq!(filtered, vec!["bob", "charlie"]);
    }

    #[test]
    fn test_filter_self_mentions_case_insensitive() {
        let mentions = vec!["Alice".to_string(), "bob".to_string()];
        let filtered = filter_self_mentions(mentions, "ALICE");
        assert_eq!(filtered, vec!["bob"]);
    }
}
