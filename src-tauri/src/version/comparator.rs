use super::parser::{parse_version, ParsedVersion};

/// 版本比较结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// 远程版本更新 (latest > local)
    Greater,
    /// 版本相同 (latest == local)
    Equal,
    /// 本地版本更新 (latest < local)，罕见但可能
    Less,
    /// 无法比较（缺少版本信息或格式不兼容）
    Unknown,
}

/// 智能版本比较
///
/// 优先使用 semver 进行语义化比较，对于非标准版本格式 fallback 到字符串比较
pub fn compare_versions(latest: &str, local: &Option<String>) -> VersionComparison {
    let Some(local_ver) = local else {
        return VersionComparison::Unknown;
    };

    let latest_parsed = parse_version(latest);
    let local_parsed = parse_version(local_ver);

    match (latest_parsed, local_parsed) {
        // 两者都是语义化版本，使用 semver 比较
        (ParsedVersion::Semantic(l), ParsedVersion::Semantic(r)) => match l.cmp(&r) {
            std::cmp::Ordering::Greater => VersionComparison::Greater,
            std::cmp::Ordering::Equal => VersionComparison::Equal,
            std::cmp::Ordering::Less => VersionComparison::Less,
        },
        // 两者都是非语义化版本，使用字符串比较
        (ParsedVersion::NonSemantic(l), ParsedVersion::NonSemantic(r)) => {
            if l == r {
                VersionComparison::Equal
            } else {
                // 无法确定大小，标记为 Unknown（保守策略）
                // 但考虑到用户体验，如果不相等我们假设有更新
                VersionComparison::Greater
            }
        }
        // 混合类型，尝试字符串比较
        _ => {
            let latest_clean = super::parser::clean_version_prefix(latest);
            let local_clean = super::parser::clean_version_prefix(local_ver);
            if latest_clean == local_clean {
                VersionComparison::Equal
            } else {
                VersionComparison::Greater
            }
        }
    }
}

/// 检查是否有更新（简化接口）
///
/// 返回 true 当远程版本比本地版本新
pub fn has_update(latest: &str, local: &Option<String>) -> bool {
    matches!(compare_versions(latest, local), VersionComparison::Greater)
}

/// 检查版本是否为预发布版本
///
/// 预发布版本包含 alpha、beta、rc 等标识
pub fn is_prerelease(version: &str) -> bool {
    if let ParsedVersion::Semantic(v) = parse_version(version) {
        !v.pre.is_empty()
    } else {
        // 对于非 semver 格式，检查常见的预发布标识
        let lower = version.to_lowercase();
        lower.contains("alpha")
            || lower.contains("beta")
            || lower.contains("rc")
            || lower.contains("preview")
            || lower.contains("canary")
            || lower.contains("nightly")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_greater() {
        // 1.10.0 > 1.9.0 (语义化比较，非字符串比较)
        assert!(has_update("1.10.0", &Some("1.9.0".to_string())));
        assert!(has_update("2.0.0", &Some("1.9.9".to_string())));
        assert!(has_update("1.0.1", &Some("1.0.0".to_string())));
    }

    #[test]
    fn test_semver_equal() {
        assert!(!has_update("1.0.0", &Some("1.0.0".to_string())));
        assert!(!has_update("v1.0.0", &Some("1.0.0".to_string())));
        assert!(!has_update("1.0.0", &Some("v1.0.0".to_string())));
    }

    #[test]
    fn test_semver_less() {
        // 本地版本更新（不应触发更新提示）
        assert!(!has_update("1.0.0", &Some("1.0.1".to_string())));
        assert!(!has_update("1.9.0", &Some("1.10.0".to_string())));
    }

    #[test]
    fn test_version_with_v_prefix() {
        assert!(has_update("v2.0.0", &Some("v1.0.0".to_string())));
        assert!(has_update("v2.0.0", &Some("1.0.0".to_string())));
        assert!(!has_update("v1.0.0", &Some("v1.0.0".to_string())));
    }

    #[test]
    fn test_no_local_version() {
        assert!(!has_update("1.0.0", &None));
        assert_eq!(
            compare_versions("1.0.0", &None),
            VersionComparison::Unknown
        );
    }

    #[test]
    fn test_non_semver_equal() {
        assert_eq!(
            compare_versions("2024-01-15", &Some("2024-01-15".to_string())),
            VersionComparison::Equal
        );
    }

    #[test]
    fn test_prerelease_detection() {
        assert!(is_prerelease("1.0.0-alpha.1"));
        assert!(is_prerelease("1.0.0-beta"));
        assert!(is_prerelease("1.0.0-rc.1"));
        assert!(is_prerelease("2.0.0-preview"));
        assert!(!is_prerelease("1.0.0"));
        assert!(!is_prerelease("v2.0.0"));
    }

    #[test]
    fn test_prerelease_comparison() {
        // 正式版 > 预发布版
        assert!(has_update("1.0.0", &Some("1.0.0-alpha.1".to_string())));
        assert!(has_update("1.0.0", &Some("1.0.0-beta".to_string())));
        // 预发布版本之间的比较
        assert!(has_update("1.0.0-beta", &Some("1.0.0-alpha".to_string())));
    }

    #[test]
    fn test_two_part_version() {
        assert!(has_update("1.10", &Some("1.9".to_string())));
        assert!(!has_update("1.9", &Some("1.10".to_string())));
    }
}
