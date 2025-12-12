use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct CrateCrate {
    max_version: String,
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    crate_info: CrateCrate,
}

/// 获取 crates.io 上的 crate 最新版本
pub async fn get_latest_version(
    crate_name: &str,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    let client = Client::new();
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);

    let response = client
        .get(&url)
        .header("User-Agent", "app-version-gui (https://github.com/try-to-fly)")
        .send()
        .await
        .map_err(|e| format!("crates.io request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("crates.io API error: {}", response.status()));
    }

    let crate_response: CrateResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse crates.io response: {}", e))?;

    let latest_version = crate_response.crate_info.max_version;
    let updated_at = crate_response
        .crate_info
        .updated_at
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Ok((latest_version, updated_at))
}
