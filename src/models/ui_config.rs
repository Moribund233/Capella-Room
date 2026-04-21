//! 用户 UI 配置模型
//!
//! 定义用户界面配置的数据结构，支持云端同步

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 用户 UI 配置数据库模型
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserUIConfig {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_config: Option<serde_json::Value>,
    pub theme_config: Option<serde_json::Value>,
    pub sidebar_config: Option<serde_json::Value>,
    pub quickbar_config: Option<serde_json::Value>,
    pub dock_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 应用配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// 主题配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// 侧边栏菜单项配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarItemConfig {
    pub name: String,
    pub icon: String,
    pub path: String,
}

/// 侧边栏配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SidebarConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<SidebarItemConfig>>,
}

/// QuickBar 子菜单项配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickChildItemConfig {
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// QuickBar 项目配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickItemConfig {
    pub key: String,
    pub display: String, // "visible" | "dropdown"
    #[serde(rename = "type")]
    pub item_type: String, // "action" | "menu"
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_alt: Option<String>,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<QuickChildItemConfig>>,
}

/// Dock 页面配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DockPageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>, // "bottom" | "left" | "right"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<DockItemConfig>>,
}

/// Dock 项目配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockItemConfig {
    pub key: String,
    pub label: String,
    pub icon: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
}

/// Dock 配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DockConfig {
    #[serde(flatten)]
    pub pages: std::collections::HashMap<String, DockPageConfig>,
}

/// 完整 UI 配置响应
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIConfigResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar: Option<SidebarConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quickbar: Option<Vec<QuickItemConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dock: Option<DockConfig>,
}

/// 保存 UI 配置请求
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SaveUIConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar: Option<SidebarConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quickbar: Option<Vec<QuickItemConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dock: Option<DockConfig>,
}

impl UserUIConfig {
    /// 转换为 UI 配置响应
    pub fn to_response(&self) -> UIConfigResponse {
        UIConfigResponse {
            app: self.app_config.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()),
            theme: self.theme_config.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()),
            sidebar: self.sidebar_config.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()),
            quickbar: self.quickbar_config.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()),
            dock: self.dock_config.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()),
        }
    }
}
