use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct PyPIInfo {
    version: String,
}

#[derive(Deserialize, Clone)]
struct PyPIRelease {
    upload_time: Option<String>,
}

#[derive(Deserialize)]
struct PyPIPackage {
    info: PyPIInfo,
    releases: Option<HashMap<String, Vec<PyPIRelease>>>,
}

/// 获取 PyPI 包的最新版本
pub async fn get_latest_version(
    package_name: &str,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    let client = Client::new();
    let url = format!("https://pypi.org/pypi/{}/json", package_name);

    let response = client
        .get(&url)
        .header("User-Agent", "app-version-gui")
        .send()
        .await
        .map_err(|e| format!("PyPI request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("PyPI API error: {}", response.status()));
    }

    let package: PyPIPackage = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse PyPI response: {}", e))?;

    let latest_version = package.info.version;

    // PyPI 使用的时间格式是 "2024-01-15T10:30:00"（不带时区）
    let published_at = package
        .releases
        .and_then(|releases| releases.get(&latest_version).cloned())
        .and_then(|releases| releases.into_iter().next())
        .and_then(|release| release.upload_time)
        .and_then(|s| {
            // 尝试解析 ISO 8601 格式
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
                .or_else(|| {
                    // 尝试解析不带时区的格式
                    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
                        .map(|dt| dt.and_utc())
                        .ok()
                })
        });

    Ok((latest_version, published_at))
}
