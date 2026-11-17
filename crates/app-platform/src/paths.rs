//! Well-known application directories.

use app_errors::{AppError, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

/// Resolved, per-user application directories.
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub crash_dir: PathBuf,
    pub export_dir: PathBuf,
    /// True when directories were redirected with `OPENRUST_HOME` (portable / test mode).
    pub portable: bool,
}

impl AppPaths {
    /// OS-conventional locations (XDG on Linux, `Library` on macOS, `AppData` on Windows).
    /// `OPENRUST_HOME` redirects everything below a single directory.
    pub fn discover() -> Result<Self> {
        if let Some(home) = std::env::var_os("OPENRUST_HOME").filter(|v| !v.is_empty()) {
            return Ok(Self::under(Path::new(&home), true));
        }
        let dirs = ProjectDirs::from("dev", "robotjaol", "OpenRust")
            .ok_or_else(|| AppError::Platform("no home directory could be determined".into()))?;
        let data = dirs.data_dir().to_path_buf();
        Ok(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            log_dir: data.join("logs"),
            crash_dir: data.join("crashes"),
            export_dir: data.join("exports"),
            data_dir: data,
            portable: false,
        })
    }

    /// Everything below `root` (used by tests and portable mode).
    pub fn under(root: &Path, portable: bool) -> Self {
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            log_dir: root.join("data").join("logs"),
            crash_dir: root.join("data").join("crashes"),
            export_dir: root.join("data").join("exports"),
            portable,
        }
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn database_file(&self) -> PathBuf {
        self.data_dir.join("open-rust.db")
    }

    /// Create every directory (idempotent).
    pub fn ensure_exist(&self) -> Result<()> {
        for d in [
            &self.config_dir,
            &self.data_dir,
            &self.log_dir,
            &self.crash_dir,
            &self.export_dir,
        ] {
            std::fs::create_dir_all(d).map_err(|e| AppError::io(d, e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_places_everything_below_root() {
        let p = AppPaths::under(Path::new("/tmp/x"), true);
        for d in [
            &p.config_dir,
            &p.data_dir,
            &p.log_dir,
            &p.crash_dir,
            &p.export_dir,
        ] {
            assert!(d.starts_with("/tmp/x"));
        }
        assert!(p.database_file().ends_with("open-rust.db"));
    }

    #[test]
    fn ensure_exist_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let p = AppPaths::under(dir.path(), true);
        p.ensure_exist().unwrap();
        p.ensure_exist().unwrap();
        assert!(p.crash_dir.is_dir());
    }
}
