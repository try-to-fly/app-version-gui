use semver::Version;

/// 版本解析结果
#[derive(Debug, Clone)]
pub enum ParsedVersion {
    /// 标准 semver 版本
    Semantic(Version),
    /// 非标准版本（日期、自定义格式等）
    NonSemantic(String),
}

/// 清理版本前缀 (v1.2.3 -> 1.2.3)
pub fn clean_version_prefix(version: &str) -> String {
    version.trim().trim_start_matches('v').trim().to_string()
}

/// 尝试解析为 semver，失败则返回原始字符串
pub fn parse_version(version: &str) -> ParsedVersion {
    let cleaned = clean_version_prefix(version);

    // 尝试直接解析
    if let Ok(v) = Version::parse(&cleaned) {
        return ParsedVersion::Semantic(v);
    }

    // 尝试宽松解析：1.2 -> 1.2.0
    if let Ok(v) = Version::parse(&format!("{}.0", cleaned)) {
        return ParsedVersion::Semantic(v);
    }

    // 尝试更宽松的解析：1 -> 1.0.0（仅当是纯数字时）
    if cleaned.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(v) = Version::parse(&format!("{}.0.0", cleaned)) {
            return ParsedVersion::Semantic(v);
        }
    }

    // 处理带有额外后缀的版本号，如 "1.2.3_1" 或 "1.2.3.4"
    // 但排除日期格式 (2024-01-15)
    if !cleaned.contains('-') || cleaned.matches('-').count() <= 1 {
        let parts: Vec<&str> = cleaned.split(|c| c == '.' || c == '_').collect();
        if parts.len() >= 3 {
            if let (Ok(major), Ok(minor), Ok(patch)) = (
                parts[0].parse::<u64>(),
                parts[1].parse::<u64>(),
                parts[2].split(|c: char| !c.is_ascii_digit()).next().unwrap_or("0").parse::<u64>(),
            ) {
                // 检查是否看起来像日期格式 (年份 > 1000)
                if major < 1000 || (parts.len() == 3 && minor <= 12 && patch <= 31) {
                    // 不是日期格式，尝试作为 semver
                    if major < 1000 {
                        if let Ok(v) = Version::parse(&format!("{}.{}.{}", major, minor, patch)) {
                            return ParsedVersion::Semantic(v);
                        }
                    }
                }
            }
        }
    }

    ParsedVersion::NonSemantic(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_version_prefix() {
        assert_eq!(clean_version_prefix("v1.2.3"), "1.2.3");
        assert_eq!(clean_version_prefix("1.2.3"), "1.2.3");
        assert_eq!(clean_version_prefix("  v1.2.3  "), "1.2.3");
    }

    #[test]
    fn test_parse_standard_semver() {
        match parse_version("1.2.3") {
            ParsedVersion::Semantic(v) => {
                assert_eq!(v.major, 1);
                assert_eq!(v.minor, 2);
                assert_eq!(v.patch, 3);
            }
            _ => panic!("Expected Semantic version"),
        }
    }

    #[test]
    fn test_parse_with_v_prefix() {
        match parse_version("v2.0.0") {
            ParsedVersion::Semantic(v) => {
                assert_eq!(v.major, 2);
                assert_eq!(v.minor, 0);
                assert_eq!(v.patch, 0);
            }
            _ => panic!("Expected Semantic version"),
        }
    }

    #[test]
    fn test_parse_two_part_version() {
        match parse_version("1.2") {
            ParsedVersion::Semantic(v) => {
                assert_eq!(v.major, 1);
                assert_eq!(v.minor, 2);
                assert_eq!(v.patch, 0);
            }
            _ => panic!("Expected Semantic version"),
        }
    }

    #[test]
    fn test_parse_prerelease() {
        match parse_version("1.0.0-alpha.1") {
            ParsedVersion::Semantic(v) => {
                assert!(!v.pre.is_empty());
            }
            _ => panic!("Expected Semantic version"),
        }
    }

    #[test]
    fn test_parse_non_semver() {
        match parse_version("2024-01-15") {
            ParsedVersion::NonSemantic(s) => {
                assert_eq!(s, "2024-01-15");
            }
            _ => panic!("Expected NonSemantic version"),
        }
    }
}
