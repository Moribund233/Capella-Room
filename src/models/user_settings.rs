use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
        }
    }
}

/// 隐私设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacySettings {
    pub online_status_visibility: String, // everyone / friends / nobody
    pub profile_visibility: String,       // everyone / friends / nobody
    pub allow_stranger_message: bool,
    pub allow_room_invitation: bool,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            online_status_visibility: "everyone".to_string(),
            profile_visibility: "everyone".to_string(),
            allow_stranger_message: true,
            allow_room_invitation: true,
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
    pub time_format: String,  // 12h / 24h
    pub date_format: String,  // YYYY-MM-DD / DD/MM/YYYY / MM/DD/YYYY
    pub first_day_of_week: String, // monday / sunday
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            time_format: "24h".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            first_day_of_week: "monday".to_string(),
        }
    }
}

/// 无障碍设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AccessibilitySettings {
    pub font_size: String,  // small / medium / large
    pub reduce_motion: bool,
    pub high_contrast: bool,
    pub dense_mode: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            font_size: "medium".to_string(),
            reduce_motion: false,
            high_contrast: false,
            dense_mode: false,
        }
    }
}

/// 媒体与存储设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaSettings {
    pub auto_download_media: bool,
    pub save_media_gallery: bool,
    pub image_quality: String, // original / high / medium / low
    pub auto_play_video: bool,
    pub auto_play_audio: bool,
}

impl Default for MediaSettings {
    fn default() -> Self {
        Self {
            auto_download_media: true,
            save_media_gallery: false,
            image_quality: "high".to_string(),
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
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRoomSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_preference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_color: Option<String>,
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
        assert_eq!(s.online_status_visibility, "everyone");
        assert_eq!(s.profile_visibility, "everyone");
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
        assert_eq!(s.time_format, "24h");
        assert_eq!(s.date_format, "YYYY-MM-DD");
        assert_eq!(s.first_day_of_week, "monday");
    }

    #[test]
    fn test_accessibility_defaults() {
        let s = AccessibilitySettings::default();
        assert_eq!(s.font_size, "medium");
        assert!(!s.reduce_motion);
        assert!(!s.high_contrast);
        assert!(!s.dense_mode);
    }

    #[test]
    fn test_media_defaults() {
        let s = MediaSettings::default();
        assert!(s.auto_download_media);
        assert!(!s.save_media_gallery);
        assert_eq!(s.image_quality, "high");
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
            "nobody"
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
}
