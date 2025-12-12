use std::sync::Mutex;
use tauri::Manager;

mod cache;
mod commands;
mod database;
mod models;
mod services;

use cache::CacheManager;
use database::Database;
use models::AppSettings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Get app data directory for database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("app_version.db");
            let db = Database::new(&db_path).expect("Failed to initialize database");

            // Load settings from database
            let settings = db.get_settings().unwrap_or_default();

            // Initialize cache with TTL from settings
            let cache = CacheManager::new(settings.cache.ttl_minutes as i64);

            app.manage(Mutex::new(db));
            app.manage(cache);
            app.manage(settings);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_all_softwares,
            commands::add_software,
            commands::update_software,
            commands::delete_software,
            commands::toggle_software,
            commands::check_version,
            commands::check_all_versions,
            commands::clear_cache,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
