use chrono::{DateTime, Utc};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::cache::CacheState;
use crate::database::DbState;
use crate::models::{AppSettings, Software, SoftwareFormData, SourceType, VersionCheckResult};
use crate::scheduler::SchedulerState;
use crate::services::{cargo, github, homebrew, local_version, npm, pypi};
use crate::version::comparator;

// Software CRUD Commands

#[tauri::command]
pub async fn get_all_softwares(db: State<'_, DbState>) -> Result<Vec<Software>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_all_softwares().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_software(
    form: SoftwareFormData,
    db: State<'_, DbState>,
    cache: State<'_, CacheState>,
    settings: State<'_, AppSettings>,
) -> Result<Software, String> {
    // 1. 先尝试获取版本信息（验证数据源有效性）
    let github_token = settings.github_token.as_deref();
    let (latest_version, published_at) = match form.source.source_type {
        SourceType::GithubRelease => {
            github::get_latest_release(&form.source.identifier, github_token).await?
        }
        SourceType::GithubTags => {
            github::get_latest_tag(&form.source.identifier, github_token).await?
        }
        SourceType::Homebrew => {
            let version = homebrew::get_version(&form.source.identifier).await?;
            (version, None)
        }
        SourceType::Npm => {
            npm::get_latest_version(&form.source.identifier).await?
        }
        SourceType::Pypi => {
            pypi::get_latest_version(&form.source.identifier).await?
        }
        SourceType::Cargo => {
            cargo::get_latest_version(&form.source.identifier).await?
        }
    };

    // 2. 获取本地版本（如果配置了）
    let local_version = form.local_version_config.as_ref().and_then(|config| {
        local_version::get_version(&config.command, config.version_arg.as_deref()).ok()
    });

    // 3. 版本获取成功，创建软件记录
    let software = Software {
        id: Uuid::new_v4().to_string(),
        name: form.name,
        source: form.source,
        local_version_config: form.local_version_config,
        latest_version: Some(latest_version.clone()),
        local_version,
        published_at,
        last_checked_at: Some(Utc::now()),
        enabled: true,
        last_notified_version: None,
        last_notified_at: None,
    };

    // 4. 插入数据库
    let db = db.lock().map_err(|e| e.to_string())?;
    db.insert_software(&software).map_err(|e| e.to_string())?;

    // 5. 更新缓存
    cache.set(&software.id, latest_version, published_at);

    Ok(software)
}

#[tauri::command]
pub async fn update_software(
    id: String,
    form: SoftwareFormData,
    db: State<'_, DbState>,
) -> Result<Software, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let existing = db
        .get_software(&id)
        .map_err(|e| e.to_string())?
        .ok_or("Software not found")?;

    let software = Software {
        id: existing.id,
        name: form.name,
        source: form.source,
        local_version_config: form.local_version_config,
        latest_version: existing.latest_version,
        local_version: existing.local_version,
        published_at: existing.published_at,
        last_checked_at: existing.last_checked_at,
        enabled: existing.enabled,
        last_notified_version: existing.last_notified_version,
        last_notified_at: existing.last_notified_at,
    };

    db.update_software(&software).map_err(|e| e.to_string())?;
    Ok(software)
}

#[tauri::command]
pub async fn delete_software(id: String, db: State<'_, DbState>) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.delete_software(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_software(id: String, enabled: bool, db: State<'_, DbState>) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut software = db
        .get_software(&id)
        .map_err(|e| e.to_string())?
        .ok_or("Software not found")?;

    software.enabled = enabled;
    db.update_software(&software).map_err(|e| e.to_string())
}

// Version Check Commands

#[tauri::command]
pub async fn check_version(
    id: String,
    force_refresh: bool,
    db: State<'_, DbState>,
    cache: State<'_, CacheState>,
    settings: State<'_, AppSettings>,
) -> Result<VersionCheckResult, String> {
    let software = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.get_software(&id)
            .map_err(|e| e.to_string())?
            .ok_or("Software not found")?
    };

    // Check cache first
    if !force_refresh {
        if let Some(cached) = cache.get(&id) {
            let local_version = get_local_version(&software);
            let has_update = comparator::has_update(&cached.latest_version, &local_version);
            return Ok(VersionCheckResult {
                software_id: id,
                latest_version: cached.latest_version,
                local_version,
                published_at: cached.published_at,
                has_update,
            });
        }
    }

    // Fetch from remote
    let github_token = settings.github_token.as_deref();
    let (latest_version, published_at) = match software.source.source_type {
        SourceType::GithubRelease => {
            github::get_latest_release(&software.source.identifier, github_token).await?
        }
        SourceType::GithubTags => {
            github::get_latest_tag(&software.source.identifier, github_token).await?
        }
        SourceType::Homebrew => {
            let version = homebrew::get_version(&software.source.identifier).await?;
            (version, None)
        }
        SourceType::Npm => {
            npm::get_latest_version(&software.source.identifier).await?
        }
        SourceType::Pypi => {
            pypi::get_latest_version(&software.source.identifier).await?
        }
        SourceType::Cargo => {
            cargo::get_latest_version(&software.source.identifier).await?
        }
    };

    // Get local version
    let local_version = get_local_version(&software);

    // Update cache
    cache.set(&id, latest_version.clone(), published_at);

    // Update database
    {
        let db = db.lock().map_err(|e| e.to_string())?;
        let mut updated_software = software.clone();
        updated_software.latest_version = Some(latest_version.clone());
        updated_software.local_version = local_version.clone();
        updated_software.published_at = published_at;
        updated_software.last_checked_at = Some(Utc::now());
        db.update_software(&updated_software).map_err(|e| e.to_string())?;
    }

    let has_update = comparator::has_update(&latest_version, &local_version);

    Ok(VersionCheckResult {
        software_id: id,
        latest_version,
        local_version,
        published_at,
        has_update,
    })
}

#[tauri::command]
pub async fn check_all_versions(
    db: State<'_, DbState>,
    cache: State<'_, CacheState>,
    settings: State<'_, AppSettings>,
) -> Result<Vec<VersionCheckResult>, String> {
    let softwares = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.get_all_softwares().map_err(|e| e.to_string())?
    };

    // 获取配置信息
    let github_token = settings.github_token.clone();

    // 过滤启用的软件
    let enabled_softwares: Vec<_> = softwares.into_iter().filter(|s| s.enabled).collect();

    if enabled_softwares.is_empty() {
        return Ok(Vec::new());
    }

    // 先检查缓存，分离出需要远程获取的软件
    let mut cached_results = Vec::new();
    let mut need_fetch = Vec::new();

    for software in enabled_softwares {
        if let Some(cached) = cache.get(&software.id) {
            let local_version = get_local_version(&software);
            let has_update = comparator::has_update(&cached.latest_version, &local_version);
            cached_results.push(VersionCheckResult {
                software_id: software.id.clone(),
                latest_version: cached.latest_version,
                local_version,
                published_at: cached.published_at,
                has_update,
            });
        } else {
            need_fetch.push(software);
        }
    }

    // 如果没有需要获取的软件，直接返回缓存结果
    if need_fetch.is_empty() {
        return Ok(cached_results);
    }

    // 并发数限制：避免 API 速率限制
    // GitHub: 60次/小时（未认证）、5000次/小时（认证）
    let max_concurrent = 5;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // 创建所有远程获取任务
    let tasks: Vec<_> = need_fetch
        .into_iter()
        .map(|software| {
            let sem = semaphore.clone();
            let token = github_token.clone();

            async move {
                // 获取信号量许可
                let _permit = sem.acquire().await.map_err(|e| e.to_string())?;

                // 从远程获取版本
                let fetch_result = fetch_remote_version(&software, token.as_deref()).await;

                // 获取本地版本
                let local_version = get_local_version(&software);

                match fetch_result {
                    Ok((latest_version, published_at)) => {
                        let has_update = comparator::has_update(&latest_version, &local_version);
                        Ok((
                            software.id.clone(),
                            VersionCheckResult {
                                software_id: software.id,
                                latest_version,
                                local_version,
                                published_at,
                                has_update,
                            },
                        ))
                    }
                    Err(e) => Err(format!("Error checking {}: {}", software.name, e)),
                }
            }
        })
        .collect();

    // 并发执行所有任务
    let results = futures::future::join_all(tasks).await;

    // 收集成功的结果并更新缓存
    let mut all_results = cached_results;
    for result in results {
        match result {
            Ok((id, check_result)) => {
                // 更新缓存
                cache.set(
                    &id,
                    check_result.latest_version.clone(),
                    check_result.published_at,
                );
                all_results.push(check_result);
            }
            Err(e) => eprintln!("{}", e),
        }
    }

    // 批量更新数据库
    {
        let db = db.lock().map_err(|e| e.to_string())?;
        for result in &all_results {
            if let Ok(Some(mut software)) = db.get_software(&result.software_id) {
                software.latest_version = Some(result.latest_version.clone());
                software.local_version = result.local_version.clone();
                software.published_at = result.published_at;
                software.last_checked_at = Some(Utc::now());
                let _ = db.update_software(&software);
            }
        }
    }

    Ok(all_results)
}

/// 从远程获取版本信息
async fn fetch_remote_version(
    software: &Software,
    github_token: Option<&str>,
) -> Result<(String, Option<DateTime<Utc>>), String> {
    match software.source.source_type {
        SourceType::GithubRelease => {
            github::get_latest_release(&software.source.identifier, github_token).await
        }
        SourceType::GithubTags => {
            github::get_latest_tag(&software.source.identifier, github_token).await
        }
        SourceType::Homebrew => {
            let version = homebrew::get_version(&software.source.identifier).await?;
            Ok((version, None))
        }
        SourceType::Npm => {
            npm::get_latest_version(&software.source.identifier).await
        }
        SourceType::Pypi => {
            pypi::get_latest_version(&software.source.identifier).await
        }
        SourceType::Cargo => {
            cargo::get_latest_version(&software.source.identifier).await
        }
    }
}

#[tauri::command]
pub async fn clear_cache(cache: State<'_, CacheState>) -> Result<(), String> {
    cache.clear();
    Ok(())
}

// Settings Commands

#[tauri::command]
pub async fn get_settings(db: State<'_, DbState>) -> Result<AppSettings, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    new_settings: AppSettings,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.save_settings(&new_settings).map_err(|e| e.to_string())
}

// Helper functions

fn get_local_version(software: &Software) -> Option<String> {
    software.local_version_config.as_ref().and_then(|config| {
        local_version::get_version(&config.command, config.version_arg.as_deref()).ok()
    })
}

// Scheduler Commands

#[tauri::command]
pub async fn update_scheduler(
    enabled: bool,
    interval_minutes: u32,
    scheduler: State<'_, SchedulerState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut scheduler = scheduler.lock().await;

    if enabled && interval_minutes > 0 {
        scheduler.restart(interval_minutes, app_handle);
        println!("[Scheduler] Updated: enabled with {} minute interval", interval_minutes);
    } else {
        scheduler.stop();
        println!("[Scheduler] Updated: disabled");
    }

    Ok(())
}
