//! Layered, versioned configuration.
//!
//! Resolution order (later wins): built-in defaults, environment profile
//! (`development` / `production`), the user's `config.toml`, then `OPENRUST_*`
//! environment variables. Files are migrated forward through [`migrate`] and are
//! always written atomically.

use app_errors::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

/// Version written to new configuration files.
pub const CURRENT_SCHEMA: u32 = 2;

/// Which deployment profile the application runs under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    #[default]
    Production,
}

impl Environment {
    /// `OPENRUST_ENV=development|production`, otherwise decided by the build profile.
    pub fn detect() -> Self {
        match std::env::var("OPENRUST_ENV").ok().as_deref() {
            Some("development" | "dev") => Self::Development,
            Some("production" | "prod") => Self::Production,
            _ if cfg!(debug_assertions) => Self::Development,
            _ => Self::Production,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    #[default]
    Indigo,
    Teal,
    Amber,
    Rose,
    Emerald,
}

impl Accent {
    pub const ALL: [Accent; 5] = [
        Accent::Indigo,
        Accent::Teal,
        Accent::Amber,
        Accent::Rose,
        Accent::Emerald,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Accent::Indigo => "Indigo",
            Accent::Teal => "Teal",
            Accent::Amber => "Amber",
            Accent::Rose => "Rose",
            Accent::Emerald => "Emerald",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    pub language: String,
    pub restore_window: bool,
    pub show_welcome: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            language: "en".into(),
            restore_window: true,
            show_welcome: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub theme: ThemeMode,
    pub accent: Accent,
    pub density: Density,
    /// UI zoom factor, clamped to `0.8..=1.6`.
    pub ui_scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            accent: Accent::Indigo,
            density: Density::Comfortable,
            ui_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 800.0,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Files {
    pub recent: Vec<String>,
    pub max_file_size_mb: u32,
}

impl Default for Files {
    fn default() -> Self {
        Self {
            recent: Vec::new(),
            max_file_size_mb: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Logging {
    /// An `EnvFilter` directive such as `info` or `open_rust=debug,warn`.
    pub level: String,
    pub to_file: bool,
}

impl Default for Logging {
    fn default() -> Self {
        Self {
            level: "info".into(),
            to_file: true,
        }
    }
}

/// The full, persisted configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub schema_version: u32,
    pub general: General,
    pub appearance: Appearance,
    pub window: WindowState,
    pub files: Files,
    pub logging: Logging,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA,
            general: General::default(),
            appearance: Appearance::default(),
            window: WindowState::default(),
            files: Files::default(),
            logging: Logging::default(),
        }
    }
}

impl Config {
    /// Defaults adjusted for the deployment profile.
    pub fn for_environment(env: Environment) -> Self {
        let mut c = Self::default();
        if env == Environment::Development {
            c.logging.level = "debug".into();
        }
        c
    }

    /// Clamp values to safe ranges. Called after every load and before every save.
    pub fn sanitize(&mut self) {
        self.appearance.ui_scale = if self.appearance.ui_scale.is_finite() {
            self.appearance.ui_scale.clamp(0.8, 1.6)
        } else {
            1.0
        };
        self.window.width = self.window.width.clamp(640.0, 7680.0);
        self.window.height = self.window.height.clamp(480.0, 4320.0);
        self.files.max_file_size_mb = self.files.max_file_size_mb.clamp(1, 512);
        self.files.recent.truncate(10);
        if self.general.language.trim().is_empty() {
            self.general.language = "en".into();
        }
    }

    /// Apply `OPENRUST_LOG` / `OPENRUST_THEME` overrides.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(level) = std::env::var("OPENRUST_LOG")
            && !level.trim().is_empty()
        {
            self.logging.level = level;
        }
        if let Ok(theme) = std::env::var("OPENRUST_THEME") {
            match theme.as_str() {
                "light" => self.appearance.theme = ThemeMode::Light,
                "dark" => self.appearance.theme = ThemeMode::Dark,
                "system" => self.appearance.theme = ThemeMode::System,
                _ => {}
            }
        }
    }

    pub fn from_toml_str(text: &str) -> Result<Self> {
        let mut value: toml::Table =
            toml::from_str(text).map_err(|e| AppError::Config(format!("invalid TOML: {e}")))?;
        migrate(&mut value)?;
        let mut cfg: Config = value
            .try_into()
            .map_err(|e: toml::de::Error| AppError::Config(format!("invalid settings: {e}")))?;
        cfg.sanitize();
        Ok(cfg)
    }

    pub fn to_toml_string(&self) -> Result<String> {
        let mut c = self.clone();
        c.sanitize();
        toml::to_string_pretty(&c).map_err(|e| AppError::Config(format!("cannot serialise: {e}")))
    }

    /// Load a file. A missing file yields defaults; a corrupt file is moved aside
    /// to `config.toml.corrupt` and defaults are returned together with the error.
    pub fn load_or_default(path: &Path, env: Environment) -> (Self, Option<AppError>) {
        let base = Self::for_environment(env);
        match std::fs::read_to_string(path) {
            Ok(text) => match Self::from_toml_str(&text) {
                Ok(cfg) => (cfg, None),
                Err(err) => {
                    let backup = path.with_extension("toml.corrupt");
                    if let Err(e) = std::fs::rename(path, &backup) {
                        tracing::warn!(error = %e, "could not preserve corrupt config");
                    }
                    (base, Some(err))
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (base, None),
            Err(e) => (base, Some(AppError::io(path, e))),
        }
    }

    /// Atomically persist to `path` (write to a temp file in the same directory, then rename).
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = self.to_toml_string()?;
        let dir = path
            .parent()
            .ok_or_else(|| AppError::Config("config path has no parent".into()))?;
        std::fs::create_dir_all(dir).map_err(|e| AppError::io(dir, e))?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| AppError::io(dir, e))?;
        tmp.write_all(text.as_bytes())
            .map_err(|e| AppError::io(path, e))?;
        tmp.as_file()
            .sync_all()
            .map_err(|e| AppError::io(path, e))?;
        tmp.persist(path).map_err(|e| AppError::io(path, e.error))?;
        Ok(())
    }
}

/// Upgrade an older on-disk table to [`CURRENT_SCHEMA`] in place.
///
/// * v1 -> v2: top-level `theme` / `language` moved into `[appearance]` / `[general]`.
pub fn migrate(table: &mut toml::Table) -> Result<()> {
    let version = table
        .get("schema_version")
        .and_then(|v| v.as_integer())
        .unwrap_or(1);
    if version < 1 || version > i64::from(CURRENT_SCHEMA) {
        return Err(AppError::Config(format!(
            "unsupported schema version {version}"
        )));
    }
    if version < 2 {
        if let Some(theme) = table.remove("theme") {
            let appearance = table
                .entry("appearance")
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
            if let Some(t) = appearance.as_table_mut() {
                t.insert("theme".into(), theme);
            }
        }
        if let Some(lang) = table.remove("language") {
            let general = table
                .entry("general")
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
            if let Some(t) = general.as_table_mut() {
                t.insert("language".into(), lang);
            }
        }
    }
    table.insert(
        "schema_version".into(),
        toml::Value::Integer(i64::from(CURRENT_SCHEMA)),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn default_round_trips() {
        let cfg = Config::default();
        let text = cfg.to_toml_string().unwrap();
        assert_eq!(Config::from_toml_str(&text).unwrap(), cfg);
    }

    #[test]
    fn v1_file_is_migrated() {
        let v1 = "theme = \"light\"\nlanguage = \"id\"\n";
        let cfg = Config::from_toml_str(v1).unwrap();
        assert_eq!(cfg.appearance.theme, ThemeMode::Light);
        assert_eq!(cfg.general.language, "id");
        assert_eq!(cfg.schema_version, CURRENT_SCHEMA);
    }

    #[test]
    fn future_schema_is_rejected() {
        assert!(Config::from_toml_str("schema_version = 99").is_err());
    }

    #[test]
    fn unknown_keys_and_partial_files_use_defaults() {
        let cfg =
            Config::from_toml_str("schema_version = 2\n[appearance]\naccent = \"teal\"\n").unwrap();
        assert_eq!(cfg.appearance.accent, Accent::Teal);
        assert_eq!(cfg.general, General::default());
    }

    #[test]
    fn save_then_load_persists_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut cfg = Config::default();
        cfg.appearance.theme = ThemeMode::Light;
        cfg.appearance.accent = Accent::Rose;
        cfg.save(&path).unwrap();
        let (loaded, err) = Config::load_or_default(&path, Environment::Production);
        assert!(err.is_none());
        assert_eq!(loaded.appearance.theme, ThemeMode::Light);
        assert_eq!(loaded.appearance.accent, Accent::Rose);
    }

    #[test]
    fn corrupt_file_is_preserved_and_defaults_returned() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is = [not toml").unwrap();
        let (cfg, err) = Config::load_or_default(&path, Environment::Production);
        assert!(err.is_some());
        assert_eq!(cfg, Config::default());
        assert!(dir.path().join("config.toml.corrupt").exists());
    }

    #[test]
    fn development_profile_is_more_verbose() {
        assert_eq!(
            Config::for_environment(Environment::Development)
                .logging
                .level,
            "debug"
        );
        assert_eq!(
            Config::for_environment(Environment::Production)
                .logging
                .level,
            "info"
        );
    }

    proptest! {
        #[test]
        fn sanitize_keeps_values_in_range(scale in any::<f32>(), w in any::<f32>(), mb in any::<u32>()) {
            let mut c = Config::default();
            c.appearance.ui_scale = scale;
            c.window.width = if w.is_finite() { w } else { 0.0 };
            c.files.max_file_size_mb = mb;
            c.sanitize();
            prop_assert!((0.8..=1.6).contains(&c.appearance.ui_scale));
            prop_assert!((640.0..=7680.0).contains(&c.window.width));
            prop_assert!((1..=512).contains(&c.files.max_file_size_mb));
        }
    }
}
