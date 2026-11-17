//! Security primitives: path policy, file-name validation, bounded and atomic file IO,
//! and secret storage abstractions.
//!
//! Design rules: no `unsafe`, no custom cryptography, no secrets in plaintext files.

use app_errors::{AppError, Result};
use std::fmt;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

pub mod secrets;

/// Reject names that are unsafe on any supported OS (separators, control characters,
/// Windows reserved device names, trailing dots/spaces, or excessive length).
pub fn validate_file_name(name: &str) -> Result<()> {
    let bad = |why: &str| Err(AppError::Validation(format!("Invalid file name: {why}.")));
    if name.is_empty() {
        return bad("it is empty");
    }
    if name.len() > 255 {
        return bad("it is longer than 255 bytes");
    }
    if name == "." || name == ".." {
        return bad("it is a relative path component");
    }
    if name.chars().any(|c| {
        c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    }) {
        return bad("it contains a reserved character");
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return bad("it ends with a dot or space");
    }
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if RESERVED.contains(&stem.as_str()) {
        return bad("it is a reserved system name");
    }
    Ok(())
}

/// Confines user-supplied relative paths to a fixed root directory.
#[derive(Debug, Clone)]
pub struct PathPolicy {
    root: PathBuf,
}

impl PathPolicy {
    /// The root is canonicalised once so later comparisons are reliable.
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let root = root.canonicalize().map_err(|e| AppError::io(root, e))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve `user_path` below the root. Rejects absolute paths, `..`, prefixes and any
    /// existing symlink that points outside the root.
    pub fn resolve(&self, user_path: impl AsRef<Path>) -> Result<PathBuf> {
        let rel = user_path.as_ref();
        for comp in rel.components() {
            match comp {
                Component::Normal(part) => validate_file_name(&part.to_string_lossy())?,
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(AppError::Security(format!(
                        "path `{}` escapes the allowed directory",
                        rel.display()
                    )));
                }
            }
        }
        let joined = self.root.join(rel);
        // Canonicalise the deepest existing ancestor to defeat symlink escapes.
        let mut probe = joined.as_path();
        while !probe.exists() {
            probe = probe
                .parent()
                .ok_or_else(|| AppError::Security("path has no existing ancestor".into()))?;
        }
        let real = probe.canonicalize().map_err(|e| AppError::io(probe, e))?;
        if !real.starts_with(&self.root) {
            return Err(AppError::Security(format!(
                "path `{}` resolves outside the allowed directory",
                rel.display()
            )));
        }
        Ok(joined)
    }
}

/// Read a file but refuse to load more than `max_bytes` (protects against huge or hostile input).
pub fn read_limited(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).map_err(|e| AppError::io(path, e))?;
    let meta = file.metadata().map_err(|e| AppError::io(path, e))?;
    if !meta.is_file() {
        return Err(AppError::Validation(
            "Only regular files can be opened.".into(),
        ));
    }
    if meta.len() > max_bytes {
        return Err(AppError::Validation(format!(
            "The file is too large ({} MB). The limit is {} MB.",
            meta.len() / 1_048_576,
            max_bytes / 1_048_576
        )));
    }
    // `take` guards against files that grow between the metadata check and the read.
    let mut buf = Vec::with_capacity(meta.len() as usize);
    file.take(max_bytes + 1)
        .read_to_end(&mut buf)
        .map_err(|e| AppError::io(path, e))?;
    if buf.len() as u64 > max_bytes {
        return Err(AppError::Validation(
            "The file grew while it was being read.".into(),
        ));
    }
    Ok(buf)
}

/// Read a UTF-8 text file with a size limit.
pub fn read_text_limited(path: &Path, max_bytes: u64) -> Result<String> {
    let bytes = read_limited(path, max_bytes)?;
    String::from_utf8(bytes)
        .map_err(|_| AppError::Validation("The file is not valid UTF-8 text.".into()))
}

/// Write `bytes` to `path` atomically: temp file in the same directory, fsync, rename.
/// Readers never observe a half-written file, even after a crash.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| AppError::io(dir, e))?;
    tmp.write_all(bytes).map_err(|e| AppError::io(path, e))?;
    tmp.as_file()
        .sync_all()
        .map_err(|e| AppError::io(path, e))?;
    tmp.persist(path).map_err(|e| AppError::io(path, e.error))?;
    Ok(())
}

/// Neutralise spreadsheet formula injection in exported CSV cells.
pub fn csv_safe(cell: &str) -> String {
    let needs_guard = matches!(
        cell.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r')
    );
    let cell = if needs_guard {
        format!("'{cell}")
    } else {
        cell.to_string()
    };
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell
    }
}

/// Wrapper whose `Debug`/`Display` never reveal the inner value; use it for tokens and keys.
#[derive(Clone, PartialEq, Eq)]
pub struct Redacted<T>(pub T);

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn accepts_ordinary_names() {
        for n in ["notes.txt", "Report 2026.md", "a", ".gitignore"] {
            validate_file_name(n).unwrap();
        }
    }

    #[test]
    fn rejects_dangerous_names() {
        for n in [
            "", "..", "a/b", "a\\b", "con", "NUL.txt", "bad:name", "trail.", "x\u{0}y",
        ] {
            assert!(validate_file_name(n).is_err(), "{n:?} should be rejected");
        }
    }

    #[test]
    fn path_policy_blocks_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let p = PathPolicy::new(dir.path()).unwrap();
        assert!(p.resolve("../etc/passwd").is_err());
        assert!(p.resolve("/etc/passwd").is_err());
        assert!(p.resolve("sub/../../x").is_err());
        assert!(p.resolve("sub/file.txt").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn path_policy_blocks_symlink_escape() {
        let outside = tempfile::tempdir().unwrap();
        let inside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), inside.path().join("link")).unwrap();
        let p = PathPolicy::new(inside.path()).unwrap();
        assert!(p.resolve("link/secret.txt").is_err());
    }

    #[test]
    fn read_limited_enforces_limit() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("big.bin");
        std::fs::write(&f, vec![0u8; 2048]).unwrap();
        assert!(read_limited(&f, 1024).is_err());
        assert_eq!(read_limited(&f, 4096).unwrap().len(), 2048);
    }

    #[test]
    fn read_limited_rejects_directories() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_limited(dir.path(), 10).is_err());
    }

    #[test]
    fn atomic_write_replaces_content() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("a.txt");
        write_atomic(&f, b"one").unwrap();
        write_atomic(&f, b"two").unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "two");
        assert_eq!(
            std::fs::read_dir(dir.path()).unwrap().count(),
            1,
            "no temp files left behind"
        );
    }

    #[test]
    fn csv_guard_and_quoting() {
        assert_eq!(csv_safe("=HYPERLINK(\"x\")"), "\"'=HYPERLINK(\"\"x\"\")\"");
        assert_eq!(csv_safe("plain"), "plain");
        assert_eq!(csv_safe("a,b"), "\"a,b\"");
    }

    #[test]
    fn redacted_never_prints_value() {
        let s = Redacted("hunter2".to_string());
        assert!(!format!("{s:?}{s}").contains("hunter2"));
    }

    proptest! {
        #[test]
        fn resolve_never_escapes_root(input in "[a-zA-Z0-9_./\\\\-]{0,40}") {
            let dir = tempfile::tempdir().unwrap();
            let p = PathPolicy::new(dir.path()).unwrap();
            if let Ok(resolved) = p.resolve(&input) {
                prop_assert!(resolved.starts_with(p.root()));
            }
        }

        #[test]
        fn csv_safe_cells_never_start_with_formula_chars_unquoted(s in ".{0,30}") {
            let out = csv_safe(&s);
            let bare = out.trim_start_matches('"');
            prop_assert!(!bare.starts_with(['=', '+', '@']));
        }
    }
}
