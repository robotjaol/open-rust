//! Filesystem service used by the UI. All reads are size-limited and all writes are atomic.

use app_errors::{AppError, Result};
use app_security::{read_text_limited, validate_file_name, write_atomic};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified_unix: Option<u64>,
    pub hidden: bool,
}

/// Bounded filesystem operations.
#[derive(Debug, Clone)]
pub struct FileService {
    pub max_read_bytes: u64,
}

impl FileService {
    pub fn new(max_read_mb: u32) -> Self {
        Self {
            max_read_bytes: u64::from(max_read_mb) * 1024 * 1024,
        }
    }

    /// List a directory: folders first, then files, each group sorted case-insensitively.
    /// Entries that cannot be inspected are skipped (and logged) instead of failing the listing.
    pub fn list_dir(&self, dir: &Path, show_hidden: bool) -> Result<Vec<FileEntry>> {
        let rd = std::fs::read_dir(dir).map_err(|e| AppError::io(dir, e))?;
        let mut out = Vec::new();
        for entry in rd {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::debug!(error = %e, "skipping unreadable directory entry");
                    continue;
                }
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            let hidden = name.starts_with('.');
            if hidden && !show_hidden {
                continue;
            }
            // `symlink_metadata` does not follow links, so a link is listed as itself.
            let Ok(meta) = entry.path().symlink_metadata() else {
                continue;
            };
            let modified_unix = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            out.push(FileEntry {
                name,
                path: entry.path(),
                is_dir: meta.is_dir(),
                size: if meta.is_file() { meta.len() } else { 0 },
                modified_unix,
                hidden,
            });
        }
        out.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(out)
    }

    pub fn read_text(&self, path: &Path) -> Result<String> {
        read_text_limited(path, self.max_read_bytes)
    }

    /// Atomically write text to `dir/name` after validating the file name.
    pub fn write_text(&self, dir: &Path, name: &str, text: &str) -> Result<PathBuf> {
        validate_file_name(name)?;
        let path = dir.join(name);
        write_atomic(&path, text.as_bytes())?;
        Ok(path)
    }

    /// Create a uniquely-named temporary file that is deleted when dropped.
    pub fn temp_file(&self, suffix: &str) -> Result<tempfile::NamedTempFile> {
        tempfile::Builder::new()
            .prefix("open-rust-")
            .suffix(suffix)
            .tempfile()
            .map_err(AppError::from)
    }

    /// The user's home directory, falling back to the current directory.
    pub fn home_dir() -> PathBuf {
        directories::UserDirs::new()
            .map(|u| u.home_dir().to_path_buf())
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

/// `1.5 MB`-style size formatting (powers of 1024).
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing_sorts_folders_first_and_hides_dotfiles() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("b.txt"), "x").unwrap();
        std::fs::write(d.path().join("A.txt"), "xx").unwrap();
        std::fs::write(d.path().join(".hidden"), "").unwrap();
        std::fs::create_dir(d.path().join("zdir")).unwrap();
        let svc = FileService::new(1);
        let names: Vec<_> = svc
            .list_dir(d.path(), false)
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(names, ["zdir", "A.txt", "b.txt"]);
        assert_eq!(svc.list_dir(d.path(), true).unwrap().len(), 4);
    }

    #[test]
    fn listing_missing_dir_is_a_user_safe_error() {
        let err = FileService::new(1)
            .list_dir(Path::new("/definitely/not/here"), false)
            .unwrap_err();
        assert!(!err.user_message().contains("/definitely"));
    }

    #[test]
    fn write_then_read_text() {
        let d = tempfile::tempdir().unwrap();
        let svc = FileService::new(1);
        let p = svc.write_text(d.path(), "note.md", "hello").unwrap();
        assert_eq!(svc.read_text(&p).unwrap(), "hello");
        assert!(svc.write_text(d.path(), "../evil.txt", "x").is_err());
    }

    #[test]
    fn read_respects_limit() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("big.txt");
        std::fs::write(&p, vec![b'a'; 2 * 1024 * 1024]).unwrap();
        assert!(FileService::new(1).read_text(&p).is_err());
    }

    #[test]
    fn temp_file_is_removed_on_drop() {
        let svc = FileService::new(1);
        let path = {
            let t = svc.temp_file(".tmp").unwrap();
            t.path().to_path_buf()
        };
        assert!(!path.exists());
    }

    #[test]
    fn size_formatting() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
    }
}
