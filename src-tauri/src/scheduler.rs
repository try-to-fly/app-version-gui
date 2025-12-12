use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;

use crate::cache::CacheState;
use crate::database::DbState;
use crate::models::{AppSettings, VersionCheckResult};
use crate::version::comparator;
use crate::services::{cargo, github, homebrew, local_version, npm, pypi};
use crate::models::SourceType;
use crate::notification::manager::{should_notify, send_notification};
use chrono::Utc;
use tokio::sync::Semaphore;

pub type SchedulerState = Arc<tokio::sync::Mutex<BackgroundScheduler>>;

pub struct BackgroundScheduler {
    cancel_tx: Option<watch::Sender<bool>>,
}

impl BackgroundScheduler {
    pub fn new() -> Self {
        Self { cancel_tx: None }
    }

    pub fn start(&mut self, interval_minutes: u32, app_handle: AppHandle) {
        // 先停止已有的任务
        self.stop();

        if interval_minutes == 0 {
            return;
        }

        let (cancel_tx, cancel_rx) = watch::channel(false);
        self.cancel_tx = Some(cancel_tx);

        let interval = Duration::from_secs(interval_minutes as u64 * 60);

        tokio::spawn(async move {
            run_scheduler(interval, cancel_rx, app_handle).await;
        });

        println!("[Scheduler] Started with interval: {} minutes", interval_minutes);
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(true);
            println!("[Scheduler] Stopped");
        }
    }

    pub fn restart(&mut self, interval_minutes: u32, app_handle: AppHandle) {
        self.stop();
        self.start(interval_minutes, app_handle);
    }
}

async fn run_scheduler(interval: Duration, mut cancel_rx: watch::Receiver<bool>, app_handle: AppHandle) {
    let mut ticker = tokio::time::interval(interval);
    // 跳过第一个立即触发的 tick
    ticker.tick().await;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                println!("[Scheduler] Running scheduled version check...");
                match perform_version_check(&app_handle).await {
                    Ok(results) => {
                        println!("[Scheduler] Check completed, {} results", results.len());
                        // 通知前端更新
                        if let Err(e) = app_handle.emit("versions-updated", &results) {
                            eprintln!("[Scheduler] Failed to emit event: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("[Scheduler] Check failed: {}", e);
                    }
                }
            }
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    println!("[Scheduler] Received cancel signal");
                    break;
                }
            }
        }
    }
}

async fn perform_version_check(app_handle: &AppHandle) -> Result<Vec<VersionCheckResult>, String> {
    let db = app_handle.state::<DbState>();
    let cache = app_handle.state::<CacheState>();
    let settings = app_handle.state::<AppSettings>();

    let softwares = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.get_all_softwares().map_err(|e| e.to_string())?
    };

    let github_token = settings.github_token.clone();

    let enabled_softwares: Vec<_> = softwares.into_iter().filter(|s| s.enabled).collect();

    if enabled_softwares.is_empty() {
        return Ok(Vec::new());
    }

    // 先检查缓存
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

    if need_fetch.is_empty() {
        return Ok(cached_results);
    }

    // 并发获取远程版本
    let max_concurrent = 5;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    let tasks: Vec<_> = need_fetch
        .into_iter()
        .map(|software| {
            let sem = semaphore.clone();
            let token = github_token.clone();

            async move {
                let _permit = sem.acquire().await.map_err(|e| e.to_string())?;

                let fetch_result = fetch_remote_version(&software, token.as_deref()).await;
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

    let results = futures::future::join_all(tasks).await;

    let mut all_results = cached_results;
    for result in results {
        match result {
            Ok((id, check_result)) => {
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

    // 发送通知
    let notification_config = &settings.notification;
    if notification_config.enabled || notification_config.test_mode {
        let db = db.lock().map_err(|e| e.to_string())?;
        for result in &all_results {
            // 测试模式下对所有软件发送通知，正常模式下仅对有更新的软件发送
            if !notification_config.test_mode && !result.has_update {
                continue;
            }

            if let Ok(Some(mut software)) = db.get_software(&result.software_id) {
                let decision = should_notify(notification_config, &software, &result.latest_version);

                if decision.should_notify {
                    println!(
                        "[Scheduler] Sending notification for {}: {} (reason: {})",
                        software.name, result.latest_version, decision.reason
                    );

                    if let Err(e) = send_notification(
                        app_handle,
                        &software.name,
                        &result.latest_version,
                        result.local_version.as_deref(),
                    ) {
                        eprintln!("[Scheduler] Failed to send notification: {}", e);
                    } else {
                        // 更新通知记录
                        software.last_notified_version = Some(result.latest_version.clone());
                        software.last_notified_at = Some(Utc::now());
                        let _ = db.update_software(&software);
                    }
                } else {
                    println!(
                        "[Scheduler] Skip notification for {}: {}",
                        software.name, decision.reason
                    );
                }
            }
        }
    }

    Ok(all_results)
}

use crate::models::Software;
use chrono::DateTime;

fn get_local_version(software: &Software) -> Option<String> {
    software.local_version_config.as_ref().and_then(|config| {
        local_version::get_version(&config.command, config.version_arg.as_deref()).ok()
    })
}

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
