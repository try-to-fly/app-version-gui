use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    GithubRelease,
    GithubTags,
    Homebrew,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::GithubRelease => "github-release",
            SourceType::GithubTags => "github-tags",
            SourceType::Homebrew => "homebrew",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "github-release" => Some(SourceType::GithubRelease),
            "github-tags" => Some(SourceType::GithubTags),
            "homebrew" => Some(SourceType::Homebrew),
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

// 主题色类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeColor {
    Zinc,
    Slate,
    Stone,
    Gray,
    Neutral,
    Red,
    Rose,
    Orange,
    Green,
    Blue,
    Yellow,
    Violet,
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self::Zinc
    }
}

impl ThemeColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeColor::Zinc => "zinc",
            ThemeColor::Slate => "slate",
            ThemeColor::Stone => "stone",
            ThemeColor::Gray => "gray",
            ThemeColor::Neutral => "neutral",
            ThemeColor::Red => "red",
            ThemeColor::Rose => "rose",
            ThemeColor::Orange => "orange",
            ThemeColor::Green => "green",
            ThemeColor::Blue => "blue",
            ThemeColor::Yellow => "yellow",
            ThemeColor::Violet => "violet",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "zinc" => Some(ThemeColor::Zinc),
            "slate" => Some(ThemeColor::Slate),
            "stone" => Some(ThemeColor::Stone),
            "gray" => Some(ThemeColor::Gray),
            "neutral" => Some(ThemeColor::Neutral),
            "red" => Some(ThemeColor::Red),
            "rose" => Some(ThemeColor::Rose),
            "orange" => Some(ThemeColor::Orange),
            "green" => Some(ThemeColor::Green),
            "blue" => Some(ThemeColor::Blue),
            "yellow" => Some(ThemeColor::Yellow),
            "violet" => Some(ThemeColor::Violet),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub cache: CacheConfig,
    pub github_token: Option<String>,
    pub theme_color: ThemeColor,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            github_token: None,
            theme_color: ThemeColor::default(),
        }
    }
}
