use regex::Regex;
use validator::ValidationError;
use once_cell::sync::Lazy;

/// 用户名正则：只允许字母、数字、下划线，必须以字母开头
static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap()
});

/// 验证用户名格式
/// - 长度：3-20字符
/// - 必须以字母开头
/// - 只允许字母、数字、下划线
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.len() < 3 || username.len() > 20 {
        return Err(ValidationError::new(
            "用户名长度必须在3-20个字符之间"
        ));
    }

    if !USERNAME_REGEX.is_match(username) {
        return Err(ValidationError::new(
            "用户名必须以字母开头，只能包含字母、数字和下划线"
        ));
    }

    Ok(())
}

/// 验证密码强度
/// - 最小长度：8字符
/// - 必须包含：大写字母、小写字母、数字
/// - 可选：特殊字符
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new(
            "密码长度至少为8个字符"
        ));
    }

    if password.len() > 128 {
        return Err(ValidationError::new(
            "密码长度不能超过128个字符"
        ));
    }

    // 检查是否包含大写字母
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    // 检查是否包含小写字母
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    // 检查是否包含数字
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_upper || !has_lower || !has_digit {
        return Err(ValidationError::new(
            "密码必须包含至少一个大写字母、一个小写字母和一个数字"
        ));
    }

    Ok(())
}

/// 验证邮箱格式（更严格的验证）
pub fn validate_email_format(email: &str) -> Result<(), ValidationError> {
    if email.len() > 254 {
        return Err(ValidationError::new(
            "邮箱长度不能超过254个字符"
        ));
    }

    // 基本格式检查
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError::new("邮箱格式不正确"));
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    if local_part.is_empty() || local_part.len() > 64 {
        return Err(ValidationError::new("邮箱本地部分格式不正确"));
    }

    if domain_part.is_empty() || !domain_part.contains('.') {
        return Err(ValidationError::new("邮箱域名格式不正确"));
    }

    Ok(())
}

/// 验证聊天室名称
/// - 长度：1-50字符
/// - 不能全是空白字符
pub fn validate_room_name(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::new(
            "聊天室名称不能为空"
        ));
    }

    if name.len() > 50 {
        return Err(ValidationError::new(
            "聊天室名称不能超过50个字符"
        ));
    }

    Ok(())
}

/// 验证聊天室描述
/// - 最大长度：200字符
pub fn validate_room_description(description: &str) -> Result<(), ValidationError> {
    if description.len() > 200 {
        return Err(ValidationError::new(
            "聊天室描述不能超过200个字符"
        ));
    }

    Ok(())
}

/// 验证消息内容
/// - 不能为空（去除空白后）
/// - 最大长度：2000字符
pub fn validate_message_content(content: &str) -> Result<(), ValidationError> {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::new(
            "消息内容不能为空"
        ));
    }

    if content.len() > 2000 {
        return Err(ValidationError::new(
            "消息内容不能超过2000个字符"
        ));
    }

    Ok(())
}

/// 验证UUID格式
pub fn validate_uuid(uuid_str: &str) -> Result<(), ValidationError> {
    if uuid::Uuid::parse_str(uuid_str).is_err() {
        return Err(ValidationError::new(
            "UUID格式不正确"
        ));
    }

    Ok(())
}

/// 验证分页参数
pub fn validate_pagination(limit: i64, offset: i64) -> Result<(), ValidationError> {
    if limit <= 0 || limit > 100 {
        return Err(ValidationError::new(
            "每页数量必须在1-100之间"
        ));
    }

    if offset < 0 {
        return Err(ValidationError::new(
            "偏移量不能为负数"
        ));
    }

    Ok(())
}

/// 验证聊天室成员数量限制
pub fn validate_max_members(max_members: i32) -> Result<(), ValidationError> {
    if max_members < 2 || max_members > 1000 {
        return Err(ValidationError::new(
            "成员数量限制必须在2-1000之间"
        ));
    }

    Ok(())
}

/// 验证可选的聊天室成员数量限制（用于validator crate）
pub fn validate_max_members_option(max_members: &Option<i32>) -> Result<(), ValidationError> {
    if let Some(ref m) = max_members {
        validate_max_members(*m)?;
    }
    Ok(())
}

/// 验证可选的聊天室名称（用于validator crate）
pub fn validate_room_name_optional(name: &Option<String>) -> Result<(), ValidationError> {
    if let Some(ref n) = name {
        validate_room_name(n)?;
    }
    Ok(())
}

/// 验证可选的聊天室描述（用于validator crate）
pub fn validate_room_description_optional(desc: &Option<String>) -> Result<(), ValidationError> {
    if let Some(ref d) = desc {
        validate_room_description(d)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("testuser").is_ok());
        assert!(validate_username("Test_User123").is_ok());
        assert!(validate_username("ab").is_err()); // 太短
        assert!(validate_username("123user").is_err()); // 数字开头
        assert!(validate_username("test-user").is_err()); // 包含连字符
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(validate_password_strength("Password123").is_ok());
        assert!(validate_password_strength("weak").is_err()); // 太短
        assert!(validate_password_strength("password123").is_err()); // 缺少大写
        assert!(validate_password_strength("PASSWORD123").is_err()); // 缺少小写
    }

    #[test]
    fn test_validate_email_format() {
        assert!(validate_email_format("test@example.com").is_ok());
        assert!(validate_email_format("test@example").is_err()); // 缺少顶级域名
        assert!(validate_email_format("testexample.com").is_err()); // 缺少@
    }

    #[test]
    fn test_validate_room_name() {
        assert!(validate_room_name("Test Room").is_ok());
        assert!(validate_room_name("  ").is_err()); // 全是空白
        assert!(validate_room_name("a".repeat(51).as_str()).is_err()); // 太长
    }

    #[test]
    fn test_validate_message_content() {
        assert!(validate_message_content("Hello World").is_ok());
        assert!(validate_message_content("  ").is_err()); // 全是空白
        assert!(validate_message_content("a".repeat(2001).as_str()).is_err()); // 太长
    }

    #[test]
    fn test_validate_pagination() {
        assert!(validate_pagination(10, 0).is_ok());
        assert!(validate_pagination(0, 0).is_err()); // limit为0
        assert!(validate_pagination(101, 0).is_err()); // limit太大
        assert!(validate_pagination(10, -1).is_err()); // offset为负
    }
}
