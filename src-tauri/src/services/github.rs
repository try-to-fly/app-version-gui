use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: String,
}

#[derive(Deserialize)]
struct GithubTag {
    name: String,
}

pub async fn get_latest_release(
    repo: &str,
    token: Option<&str>,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let mut request = client
        .get(&url)
        .header("User-Agent", "app-version-gui")
        .header("Accept", "application/vnd.github.v3+json");

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let release: GithubRelease = response.json().await.map_err(|e| e.to_string())?;

    let published_at = DateTime::parse_from_rfc3339(&release.published_at)
        .ok()
        .map(|dt| dt.with_timezone(&Utc));

    Ok((release.tag_name, published_at))
}

pub async fn get_latest_tag(
    repo: &str,
    token: Option<&str>,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/tags", repo);

    let mut request = client
        .get(&url)
        .header("User-Agent", "app-version-gui")
        .header("Accept", "application/vnd.github.v3+json");

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let tags: Vec<GithubTag> = response.json().await.map_err(|e| e.to_string())?;

    let latest = tags.first().ok_or("No tags found")?;
    Ok((latest.name.clone(), None))
}
