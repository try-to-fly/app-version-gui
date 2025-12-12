use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct NpmPackageInfo {
    #[serde(rename = "dist-tags")]
    dist_tags: HashMap<String, String>,
    time: Option<HashMap<String, String>>,
}

/// 获取 npm 包的最新版本
pub async fn get_latest_version(
    package_name: &str,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    let client = Client::new();
    let url = format!("https://registry.npmjs.org/{}", package_name);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", "app-version-gui")
        .send()
        .await
        .map_err(|e| format!("npm request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("npm API error: {}", response.status()));
    }

    let package_info: NpmPackageInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse npm response: {}", e))?;

    let latest_version = package_info
        .dist_tags
        .get("latest")
        .ok_or("No 'latest' tag found")?
        .clone();

    let published_at = package_info
        .time
        .and_then(|time| time.get(&latest_version).cloned())
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Ok((latest_version, published_at))
}
