use chrono::{Timelike, Utc};

use crate::models::{NotificationConfig, Software};
use crate::version::{is_prerelease, parse_version, ParsedVersion};

/// 通知判断结果
pub struct NotificationDecision {
    pub should_notify: bool,
    pub reason: String,
}

/// 检查是否应该发送通知
pub fn should_notify(
    config: &NotificationConfig,
    software: &Software,
    new_version: &str,
) -> NotificationDecision {
    // 测试模式：跳过所有检查（包括静默时段），直接发送通知
    if config.test_mode {
        return NotificationDecision {
            should_notify: true,
            reason: "测试模式".to_string(),
        };
    }

    // 检查是否启用通知
    if !config.enabled {
        return NotificationDecision {
            should_notify: false,
            reason: "通知已禁用".to_string(),
        };
    }

    // 检查静默时段
    if is_silent_period(config) {
        return NotificationDecision {
            should_notify: false,
            reason: "当前处于静默时段".to_string(),
        };
    }

    // 检查是否已通知过此版本
    if let Some(ref last_notified) = software.last_notified_version {
        if last_notified == new_version {
            return NotificationDecision {
                should_notify: false,
                reason: "此版本已通知过".to_string(),
            };
        }
    }

    // 检查预发布版本
    if is_prerelease(new_version) && !config.notify_on_prerelease {
        return NotificationDecision {
            should_notify: false,
            reason: "预发布版本通知已禁用".to_string(),
        };
    }

    // 检查版本类型策略
    if let Some(ref old_ver) = software.latest_version {
        match check_version_type(config, old_ver, new_version) {
            Some(decision) => return decision,
            None => {}
        }
    }

    NotificationDecision {
        should_notify: true,
        reason: "版本更新".to_string(),
    }
}

/// 检查当前是否在静默时段
fn is_silent_period(config: &NotificationConfig) -> bool {
    let Some(start) = config.silent_start_hour else {
        return false;
    };
    let Some(end) = config.silent_end_hour else {
        return false;
    };

    let now_hour = Utc::now().hour() as u8;

    if start <= end {
        // 正常时段 (e.g., 8:00 - 22:00)
        now_hour >= start && now_hour < end
    } else {
        // 跨日时段 (e.g., 22:00 - 8:00)
        now_hour >= start || now_hour < end
    }
}

/// 根据版本差异类型判断是否通知
fn check_version_type(
    config: &NotificationConfig,
    old: &str,
    new: &str,
) -> Option<NotificationDecision> {
    let old_parsed = parse_version(old);
    let new_parsed = parse_version(new);

    // 仅比较语义化版本
    if let (ParsedVersion::Semantic(old_v), ParsedVersion::Semantic(new_v)) =
        (old_parsed, new_parsed)
    {
        if new_v.major > old_v.major {
            if !config.notify_on_major {
                return Some(NotificationDecision {
                    should_notify: false,
                    reason: "主版本更新通知已禁用".to_string(),
                });
            }
            return None; // 允许通知
        }

        if new_v.minor > old_v.minor {
            if !config.notify_on_minor {
                return Some(NotificationDecision {
                    should_notify: false,
                    reason: "次版本更新通知已禁用".to_string(),
                });
            }
            return None; // 允许通知
        }

        if new_v.patch > old_v.patch {
            if !config.notify_on_patch {
                return Some(NotificationDecision {
                    should_notify: false,
                    reason: "补丁版本更新通知已禁用".to_string(),
                });
            }
            return None; // 允许通知
        }
    }

    None // 其他情况默认允许通知
}

/// 发送系统通知
pub fn send_notification(
    app: &tauri::AppHandle,
    software_name: &str,
    new_version: &str,
    local_version: Option<&str>,
) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    let body = match local_version {
        Some(local) => format!(
            "{} 有新版本可用\n最新版本: {}\n当前版本: {}",
            software_name, new_version, local
        ),
        None => format!(
            "{} 有新版本可用\n最新版本: {}",
            software_name, new_version
        ),
    };

    app.notification()
        .builder()
        .title("软件更新提醒")
        .body(&body)
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> NotificationConfig {
        NotificationConfig {
            enabled: true,
            notify_on_major: true,
            notify_on_minor: true,
            notify_on_patch: false,
            notify_on_prerelease: false,
            silent_start_hour: None,
            silent_end_hour: None,
            test_mode: false,
        }
    }

    fn test_software() -> Software {
        Software {
            id: "test".to_string(),
            name: "Test".to_string(),
            source: crate::models::SourceConfig {
                source_type: crate::models::SourceType::GithubRelease,
                identifier: "test/test".to_string(),
            },
            local_version_config: None,
            latest_version: Some("1.0.0".to_string()),
            local_version: Some("1.0.0".to_string()),
            published_at: None,
            last_checked_at: None,
            enabled: true,
            last_notified_version: None,
            last_notified_at: None,
        }
    }

    #[test]
    fn test_notification_disabled() {
        let mut config = default_config();
        config.enabled = false;
        let software = test_software();

        let decision = should_notify(&config, &software, "2.0.0");
        assert!(!decision.should_notify);
    }

    #[test]
    fn test_already_notified() {
        let config = default_config();
        let mut software = test_software();
        software.last_notified_version = Some("2.0.0".to_string());

        let decision = should_notify(&config, &software, "2.0.0");
        assert!(!decision.should_notify);
    }

    #[test]
    fn test_patch_disabled() {
        let config = default_config();
        let software = test_software();

        let decision = should_notify(&config, &software, "1.0.1");
        assert!(!decision.should_notify);
    }

    #[test]
    fn test_major_enabled() {
        let config = default_config();
        let software = test_software();

        let decision = should_notify(&config, &software, "2.0.0");
        assert!(decision.should_notify);
    }

    #[test]
    fn test_minor_enabled() {
        let config = default_config();
        let software = test_software();

        let decision = should_notify(&config, &software, "1.1.0");
        assert!(decision.should_notify);
    }
}
