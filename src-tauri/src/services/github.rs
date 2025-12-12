use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: String,
}

#[derive(Deserialize)]
struct GithubTagCommit {
    sha: String,
}

#[derive(Deserialize)]
struct GithubTag {
    name: String,
    commit: GithubTagCommit,
}

#[derive(Deserialize)]
struct GithubCommitAuthor {
    date: String,
}

#[derive(Deserialize)]
struct GithubCommitDetail {
    author: GithubCommitAuthor,
}

#[derive(Deserialize)]
struct GithubCommit {
    commit: GithubCommitDetail,
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

    // 获取 commit 信息来得到 tag 创建时间
    let commit_url = format!(
        "https://api.github.com/repos/{}/commits/{}",
        repo, latest.commit.sha
    );

    let mut commit_request = client
        .get(&commit_url)
        .header("User-Agent", "app-version-gui")
        .header("Accept", "application/vnd.github.v3+json");

    if let Some(token) = token {
        commit_request = commit_request.header("Authorization", format!("Bearer {}", token));
    }

    let created_at = match commit_request.send().await {
        Ok(response) if response.status().is_success() => {
            match response.json::<GithubCommit>().await {
                Ok(commit) => DateTime::parse_from_rfc3339(&commit.commit.author.date)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc)),
                Err(_) => None,
            }
        }
        _ => None,
    };

    Ok((latest.name.clone(), created_at))
}
