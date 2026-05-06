use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================
// 枚举类型定义
// ============================================================

/// 可见性设置枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Everyone, // 所有人可见
    Friends, // 仅好友可见
    Nobody,  // 不可见
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Everyone => "everyone",
            Visibility::Friends => "friends",
            Visibility::Nobody => "nobody",
        }
    }
}

/// 时间格式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeFormat {
    #[serde(rename = "12h")]
    H12,
    #[serde(rename = "24h")]
    #[default]
    H24,
}

impl TimeFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFormat::H12 => "12h",
            TimeFormat::H24 => "24h",
        }
    }
}

/// 日期格式枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DateFormat {
    #[serde(rename = "YYYY-MM-DD")]
    #[default]
    YyyyMmDd,
    #[serde(rename = "DD/MM/YYYY")]
    DdMmYyyy,
    #[serde(rename = "MM/DD/YYYY")]
    MmDdYyyy,
}

impl DateFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DateFormat::YyyyMmDd => "YYYY-MM-DD",
            DateFormat::DdMmYyyy => "DD/MM/YYYY",
            DateFormat::MmDdYyyy => "MM/DD/YYYY",
        }
    }
}

/// 星期起始日枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FirstDayOfWeek {
    #[default]
    Monday,
    Sunday,
}

impl FirstDayOfWeek {
    pub fn as_str(&self) -> &'static str {
        match self {
            FirstDayOfWeek::Monday => "monday",
            FirstDayOfWeek::Sunday => "sunday",
        }
    }
}

/// 字体大小枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FontSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl FontSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            FontSize::Small => "small",
            FontSize::Medium => "medium",
            FontSize::Large => "large",
        }
    }
}

/// 图片质量枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageQuality {
    Original,
    #[default]
    High,
    Medium,
    Low,
}

impl ImageQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageQuality::Original => "original",
            ImageQuality::High => "high",
            ImageQuality::Medium => "medium",
            ImageQuality::Low => "low",
        }
    }
}

/// 房间通知偏好枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RoomNotificationPreference {
    #[default]
    All, // 所有通知
    MentionOnly, // 仅@提及
    Muted,       // 静音
}

impl RoomNotificationPreference {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoomNotificationPreference::All => "all",
            RoomNotificationPreference::MentionOnly => "mention_only",
            RoomNotificationPreference::Muted => "muted",
        }
    }
}

// ============================================================
// 用户整体设置
// ============================================================

/// 用户设置完整响应（合并所有 JSONB 分组）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsResponse {
    pub notification: NotificationSettings,
    pub privacy: PrivacySettings,
    pub message: MessageSettings,
    pub language: LanguageSettings,
    pub accessibility: AccessibilitySettings,
    pub media: MediaSettings,
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationSettings {
    pub private_message: bool,
    pub mentioned: bool,
    pub room_invitation: bool,
    pub system_notification: bool,
    pub file_upload_complete: bool,
    pub sound_enabled: bool,
    pub desktop_notification: bool,
    pub do_not_disturb: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            private_message: true,
            mentioned: true,
            room_invitation: true,
            system_notification: true,
            file_upload_complete: true,
            sound_enabled: true,
            desktop_notification: true,
            do_not_disturb: false,
        }
    }
}

/// 隐私设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacySettings {
    pub online_status_visibility: Visibility,
    pub profile_visibility: Visibility,
    pub allow_stranger_message: bool,
    pub allow_room_invitation: bool,
    /// 单设备登录开关：开启后只允许一个设备登录
    pub single_device_login: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            online_status_visibility: Visibility::default(),
            profile_visibility: Visibility::default(),
            allow_stranger_message: true,
            allow_room_invitation: true,
            single_device_login: false,
        }
    }
}

/// 消息设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MessageSettings {
    pub message_preview: bool,
    pub read_receipt: bool,
    pub typing_indicator: bool,
    pub do_not_disturb: bool,
}

impl Default for MessageSettings {
    fn default() -> Self {
        Self {
            message_preview: true,
            read_receipt: true,
            typing_indicator: true,
            do_not_disturb: false,
        }
    }
}

/// 语言与地区设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LanguageSettings {
    pub language: String,
    pub timezone: String,
    pub time_format: TimeFormat,
    pub date_format: DateFormat,
    pub first_day_of_week: FirstDayOfWeek,
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            time_format: TimeFormat::default(),
            date_format: DateFormat::default(),
            first_day_of_week: FirstDayOfWeek::default(),
        }
    }
}

/// 无障碍设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccessibilitySettings {
    pub font_size: FontSize,
    pub reduce_motion: bool,
    pub high_contrast: bool,
    pub dense_mode: bool,
}

/// 媒体与存储设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaSettings {
    pub auto_download_media: bool,
    pub save_media_gallery: bool,
    pub image_quality: ImageQuality,
    pub auto_play_video: bool,
    pub auto_play_audio: bool,
}

impl Default for MediaSettings {
    fn default() -> Self {
        Self {
            auto_download_media: true,
            save_media_gallery: false,
            image_quality: ImageQuality::default(),
            auto_play_video: true,
            auto_play_audio: false,
        }
    }
}

/// 部分更新用户设置请求
/// 客户端只传递需要修改的分组，未传的分组保持不变
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<NotificationSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<PrivacySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LanguageSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility: Option<AccessibilitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaSettings>,
}

// ============================================================
// 用户房间级设置
// ============================================================

/// 用户房间设置（数据库模型）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserRoomSettings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub is_muted: bool,
    pub notification_preference: String, // all / mention_only / muted
    pub is_pinned: bool,
    pub custom_name: Option<String>,
    pub custom_color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户房间设置响应
#[derive(Debug, Clone, Serialize)]
pub struct UserRoomSettingsResponse {
    pub room_id: Uuid,
    pub is_muted: bool,
    pub notification_preference: String,
    pub is_pinned: bool,
    pub custom_name: Option<String>,
    pub custom_color: Option<String>,
}

impl From<UserRoomSettings> for UserRoomSettingsResponse {
    fn from(s: UserRoomSettings) -> Self {
        Self {
            room_id: s.room_id,
            is_muted: s.is_muted,
            notification_preference: s.notification_preference,
            is_pinned: s.is_pinned,
            custom_name: s.custom_name,
            custom_color: s.custom_color,
        }
    }
}

/// 更新房间设置请求
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateRoomSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_preference: Option<RoomNotificationPreference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_color: Option<String>,
}

impl UpdateRoomSettingsRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<(), String> {
        // 验证自定义名称长度
        if let Some(ref name) = self.custom_name {
            if name.len() > 100 {
                return Err("自定义名称不能超过100个字符".to_string());
            }
        }

        // 验证颜色格式（十六进制颜色）
        if let Some(ref color) = self.custom_color {
            if !Self::is_valid_hex_color(color) {
                return Err("颜色格式无效，应为 #RRGGBB 格式".to_string());
            }
        }

        Ok(())
    }

    /// 验证十六进制颜色格式
    fn is_valid_hex_color(color: &str) -> bool {
        if color.len() != 7 {
            return false;
        }
        if !color.starts_with('#') {
            return false;
        }
        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_notification_defaults() {
        let s = NotificationSettings::default();
        assert!(s.private_message);
        assert!(s.mentioned);
        assert!(s.room_invitation);
        assert!(s.system_notification);
        assert!(s.file_upload_complete);
        assert!(s.sound_enabled);
        assert!(s.desktop_notification);
    }

    #[test]
    fn test_privacy_defaults() {
        let s = PrivacySettings::default();
        assert_eq!(s.online_status_visibility, Visibility::Everyone);
        assert_eq!(s.profile_visibility, Visibility::Everyone);
        assert!(s.allow_stranger_message);
        assert!(s.allow_room_invitation);
    }

    #[test]
    fn test_message_defaults() {
        let s = MessageSettings::default();
        assert!(s.message_preview);
        assert!(s.read_receipt);
        assert!(s.typing_indicator);
        assert!(!s.do_not_disturb);
    }

    #[test]
    fn test_language_defaults() {
        let s = LanguageSettings::default();
        assert_eq!(s.language, "zh-CN");
        assert_eq!(s.timezone, "Asia/Shanghai");
        assert_eq!(s.time_format, TimeFormat::H24);
        assert_eq!(s.date_format, DateFormat::YyyyMmDd);
        assert_eq!(s.first_day_of_week, FirstDayOfWeek::Monday);
    }

    #[test]
    fn test_accessibility_defaults() {
        let s = AccessibilitySettings::default();
        assert_eq!(s.font_size, FontSize::Medium);
        assert!(!s.reduce_motion);
        assert!(!s.high_contrast);
        assert!(!s.dense_mode);
    }

    #[test]
    fn test_media_defaults() {
        let s = MediaSettings::default();
        assert!(s.auto_download_media);
        assert!(!s.save_media_gallery);
        assert_eq!(s.image_quality, ImageQuality::High);
        assert!(s.auto_play_video);
        assert!(!s.auto_play_audio);
    }

    #[test]
    fn test_notification_roundtrip() {
        let original = NotificationSettings {
            private_message: false,
            mentioned: true,
            room_invitation: false,
            system_notification: true,
            file_upload_complete: false,
            sound_enabled: false,
            desktop_notification: true,
            do_not_disturb: false,
        };
        let json = serde_json::to_value(&original).unwrap();
        let deserialized: NotificationSettings = serde_json::from_value(json).unwrap();
        assert!(!deserialized.private_message);
        assert!(deserialized.mentioned);
        assert!(!deserialized.room_invitation);
    }

    #[test]
    fn test_partial_update_request() {
        let json = json!({
            "notification": {
                "private_message": false,
                "mentioned": false
            },
            "privacy": {
                "online_status_visibility": "nobody"
            }
        });
        let req: UpdateUserSettingsRequest = serde_json::from_value(json).unwrap();
        assert!(req.notification.is_some());
        assert!(req.privacy.is_some());
        assert!(req.message.is_none());
        assert!(req.language.is_none());
        assert!(req.accessibility.is_none());
        assert!(req.media.is_none());
        assert!(!req.notification.as_ref().unwrap().private_message);
        assert_eq!(
            req.privacy.as_ref().unwrap().online_status_visibility,
            Visibility::Nobody
        );
    }

    #[test]
    fn test_room_settings_response_roundtrip() {
        let resp = UserRoomSettingsResponse {
            room_id: Uuid::new_v4(),
            is_muted: true,
            notification_preference: "mention_only".to_string(),
            is_pinned: true,
            custom_name: Some("Test Room".to_string()),
            custom_color: Some("#ff0000".to_string()),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["is_muted"], true);
        assert_eq!(json["notification_preference"], "mention_only");
        assert_eq!(json["custom_name"], "Test Room");
    }

    #[test]
    fn test_update_room_settings_request() {
        let json = json!({
            "is_muted": true,
            "is_pinned": true,
            "custom_name": "Favorites"
        });
        let req: UpdateRoomSettingsRequest = serde_json::from_value(json).unwrap();
        assert_eq!(req.is_muted, Some(true));
        assert_eq!(req.is_pinned, Some(true));
        assert_eq!(req.custom_name, Some("Favorites".to_string()));
        assert!(req.custom_color.is_none());
        assert!(req.notification_preference.is_none());
    }

    #[test]
    fn test_room_notification_preference_enum() {
        let pref = RoomNotificationPreference::MentionOnly;
        assert_eq!(pref.as_str(), "mention_only");

        let json = serde_json::to_value(&pref).unwrap();
        assert_eq!(json, "mention_only");

        let deserialized: RoomNotificationPreference = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, RoomNotificationPreference::MentionOnly);
    }

    #[test]
    fn test_visibility_enum() {
        let vis = Visibility::Friends;
        assert_eq!(vis.as_str(), "friends");

        let json = serde_json::to_value(&vis).unwrap();
        assert_eq!(json, "friends");

        let deserialized: Visibility = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, Visibility::Friends);
    }

    #[test]
    fn test_update_room_settings_validation() {
        // 有效数据
        let req = UpdateRoomSettingsRequest {
            is_muted: Some(true),
            notification_preference: Some(RoomNotificationPreference::All),
            is_pinned: Some(false),
            custom_name: Some("Test Room".to_string()),
            custom_color: Some("#ff0000".to_string()),
        };
        assert!(req.validate().is_ok());

        // 无效颜色格式
        let req_invalid_color = UpdateRoomSettingsRequest {
            custom_color: Some("invalid".to_string()),
            ..Default::default()
        };
        assert!(req_invalid_color.validate().is_err());

        // 无效颜色 - 缺少 #
        let req_no_hash = UpdateRoomSettingsRequest {
            custom_color: Some("ff0000".to_string()),
            ..Default::default()
        };
        assert!(req_no_hash.validate().is_err());

        // 无效颜色 - 长度不对
        let req_wrong_len = UpdateRoomSettingsRequest {
            custom_color: Some("#ff00".to_string()),
            ..Default::default()
        };
        assert!(req_wrong_len.validate().is_err());
    }

    #[test]
    fn test_valid_hex_colors() {
        assert!(UpdateRoomSettingsRequest {
            custom_color: Some("#ff0000".to_string()),
            ..Default::default()
        }
        .validate()
        .is_ok());

        assert!(UpdateRoomSettingsRequest {
            custom_color: Some("#00FF00".to_string()),
            ..Default::default()
        }
        .validate()
        .is_ok());

        assert!(UpdateRoomSettingsRequest {
            custom_color: Some("#123abc".to_string()),
            ..Default::default()
        }
        .validate()
        .is_ok());
    }
}
