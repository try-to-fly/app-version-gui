use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::cache::CacheState;
use crate::database::DbState;
use crate::models::{AppSettings, Software, SoftwareFormData, SourceType, VersionCheckResult};
use crate::services::{github, homebrew, local_version};

// Software CRUD Commands

#[tauri::command]
pub async fn get_all_softwares(db: State<'_, DbState>) -> Result<Vec<Software>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_all_softwares().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_software(form: SoftwareFormData, db: State<'_, DbState>) -> Result<Software, String> {
    let software = Software {
        id: Uuid::new_v4().to_string(),
        name: form.name,
        source: form.source,
        local_version_config: form.local_version_config,
        latest_version: None,
        local_version: None,
        published_at: None,
        last_checked_at: None,
        enabled: true,
    };

    let db = db.lock().map_err(|e| e.to_string())?;
    db.insert_software(&software).map_err(|e| e.to_string())?;
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
            let has_update = check_has_update(&cached.latest_version, &local_version);
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

    let has_update = check_has_update(&latest_version, &local_version);

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

    let mut results = Vec::new();

    for software in softwares {
        if !software.enabled {
            continue;
        }

        match check_version(
            software.id.clone(),
            false,
            db.clone(),
            cache.clone(),
            settings.clone(),
        )
        .await
        {
            Ok(result) => results.push(result),
            Err(e) => eprintln!("Error checking {}: {}", software.name, e),
        }
    }

    Ok(results)
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

fn check_has_update(latest: &str, local: &Option<String>) -> bool {
    match local {
        None => false,
        Some(local_ver) => {
            let latest_clean = latest.trim_start_matches('v');
            let local_clean = local_ver.trim_start_matches('v');
            latest_clean != local_clean
        }
    }
}
