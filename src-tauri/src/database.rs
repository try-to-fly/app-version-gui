use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::sync::Mutex;

use crate::models::{AppSettings, CacheConfig, LocalVersionConfig, Software, SourceConfig, SourceType, ThemeColor};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS softwares (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_identifier TEXT NOT NULL,
                local_command TEXT,
                local_version_arg TEXT,
                latest_version TEXT,
                local_version TEXT,
                published_at TEXT,
                last_checked_at TEXT,
                enabled INTEGER DEFAULT 1
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_all_softwares(&self) -> Result<Vec<Software>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, source_type, source_identifier, local_command, local_version_arg,
                    latest_version, local_version, published_at, last_checked_at, enabled
             FROM softwares ORDER BY name"
        )?;

        let software_iter = stmt.query_map([], |row| {
            let source_type_str: String = row.get(2)?;
            let source_type = SourceType::from_str(&source_type_str)
                .unwrap_or(SourceType::GithubRelease);

            let local_command: Option<String> = row.get(4)?;
            let local_version_arg: Option<String> = row.get(5)?;
            let local_version_config = local_command.map(|cmd| LocalVersionConfig {
                command: cmd,
                version_arg: local_version_arg,
            });

            let published_at_str: Option<String> = row.get(8)?;
            let published_at = published_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let last_checked_at_str: Option<String> = row.get(9)?;
            let last_checked_at = last_checked_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(Software {
                id: row.get(0)?,
                name: row.get(1)?,
                source: SourceConfig {
                    source_type,
                    identifier: row.get(3)?,
                },
                local_version_config,
                latest_version: row.get(6)?,
                local_version: row.get(7)?,
                published_at,
                last_checked_at,
                enabled: row.get::<_, i32>(10)? != 0,
            })
        })?;

        software_iter.collect()
    }

    pub fn get_software(&self, id: &str) -> Result<Option<Software>> {
        let softwares = self.get_all_softwares()?;
        Ok(softwares.into_iter().find(|s| s.id == id))
    }

    pub fn insert_software(&self, software: &Software) -> Result<()> {
        self.conn.execute(
            "INSERT INTO softwares (id, name, source_type, source_identifier, local_command,
             local_version_arg, latest_version, local_version, published_at, last_checked_at, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                software.id,
                software.name,
                software.source.source_type.as_str(),
                software.source.identifier,
                software.local_version_config.as_ref().map(|c| &c.command),
                software.local_version_config.as_ref().and_then(|c| c.version_arg.as_ref()),
                software.latest_version,
                software.local_version,
                software.published_at.map(|dt| dt.to_rfc3339()),
                software.last_checked_at.map(|dt| dt.to_rfc3339()),
                software.enabled as i32,
            ],
        )?;
        Ok(())
    }

    pub fn update_software(&self, software: &Software) -> Result<()> {
        self.conn.execute(
            "UPDATE softwares SET name = ?2, source_type = ?3, source_identifier = ?4,
             local_command = ?5, local_version_arg = ?6, latest_version = ?7, local_version = ?8,
             published_at = ?9, last_checked_at = ?10, enabled = ?11
             WHERE id = ?1",
            params![
                software.id,
                software.name,
                software.source.source_type.as_str(),
                software.source.identifier,
                software.local_version_config.as_ref().map(|c| &c.command),
                software.local_version_config.as_ref().and_then(|c| c.version_arg.as_ref()),
                software.latest_version,
                software.local_version,
                software.published_at.map(|dt| dt.to_rfc3339()),
                software.last_checked_at.map(|dt| dt.to_rfc3339()),
                software.enabled as i32,
            ],
        )?;
        Ok(())
    }

    pub fn delete_software(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM softwares WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut settings = AppSettings::default();
        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "cache_ttl_minutes" => {
                    settings.cache.ttl_minutes = value.parse().unwrap_or(30);
                }
                "auto_refresh_enabled" => {
                    settings.cache.auto_refresh_enabled = value == "true";
                }
                "auto_refresh_interval" => {
                    settings.cache.auto_refresh_interval = value.parse().unwrap_or(60);
                }
                "github_token" => {
                    settings.github_token = Some(value);
                }
                "theme_color" => {
                    settings.theme_color = ThemeColor::from_str(&value).unwrap_or_default();
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let upsert = |key: &str, value: &str| -> Result<()> {
            self.conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
            Ok(())
        };

        upsert("cache_ttl_minutes", &settings.cache.ttl_minutes.to_string())?;
        upsert("auto_refresh_enabled", &settings.cache.auto_refresh_enabled.to_string())?;
        upsert("auto_refresh_interval", &settings.cache.auto_refresh_interval.to_string())?;
        upsert("theme_color", settings.theme_color.as_str())?;

        if let Some(ref token) = settings.github_token {
            upsert("github_token", token)?;
        }

        Ok(())
    }
}

pub type DbState = Mutex<Database>;
