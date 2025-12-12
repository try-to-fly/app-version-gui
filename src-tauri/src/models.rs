use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    GithubRelease,
    GithubTags,
    Homebrew,
    Npm,
    Pypi,
    Cargo,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::GithubRelease => "github-release",
            SourceType::GithubTags => "github-tags",
            SourceType::Homebrew => "homebrew",
            SourceType::Npm => "npm",
            SourceType::Pypi => "pypi",
            SourceType::Cargo => "cargo",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "github-release" => Some(SourceType::GithubRelease),
            "github-tags" => Some(SourceType::GithubTags),
            "homebrew" => Some(SourceType::Homebrew),
            "npm" => Some(SourceType::Npm),
            "pypi" => Some(SourceType::Pypi),
            "cargo" => Some(SourceType::Cargo),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceConfig {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalVersionConfig {
    pub command: String,
    pub version_arg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    pub id: String,
    pub name: String,
    pub source: SourceConfig,
    pub local_version_config: Option<LocalVersionConfig>,
    pub latest_version: Option<String>,
    pub local_version: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    // 通知相关字段
    #[serde(default)]
    pub last_notified_version: Option<String>,
    #[serde(default)]
    pub last_notified_at: Option<DateTime<Utc>>,
}

impl Software {
    pub fn new(id: String, name: String, source: SourceConfig) -> Self {
        Self {
            id,
            name,
            source,
            local_version_config: None,
            latest_version: None,
            local_version: None,
            published_at: None,
            last_checked_at: None,
            enabled: true,
            last_notified_version: None,
            last_notified_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareFormData {
    pub name: String,
    pub source: SourceConfig,
    pub local_version_config: Option<LocalVersionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionCheckResult {
    pub software_id: String,
    pub latest_version: String,
    pub local_version: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub has_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheConfig {
    pub ttl_minutes: u32,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_interval: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_minutes: 30,
            auto_refresh_enabled: true,
            auto_refresh_interval: 60,
        }
    }
}

// 主题模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationConfig {
    /// 是否启用通知
    pub enabled: bool,
    /// 主版本更新时通知
    pub notify_on_major: bool,
    /// 次版本更新时通知
    pub notify_on_minor: bool,
    /// 补丁版本更新时通知
    pub notify_on_patch: bool,
    /// 预发布版本更新时通知
    pub notify_on_prerelease: bool,
    /// 静默时段开始小时 (0-23)
    pub silent_start_hour: Option<u8>,
    /// 静默时段结束小时 (0-23)
    pub silent_end_hour: Option<u8>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notify_on_major: true,
            notify_on_minor: true,
            notify_on_patch: false,
            notify_on_prerelease: false,
            silent_start_hour: Some(22),
            silent_end_hour: Some(8),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub cache: CacheConfig,
    pub github_token: Option<String>,
    pub theme: ThemeMode,
    /// 通知配置
    #[serde(default)]
    pub notification: NotificationConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            github_token: None,
            theme: ThemeMode::default(),
            notification: NotificationConfig::default(),
        }
    }
}
