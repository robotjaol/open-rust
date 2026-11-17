//! Host and build information (shown on the About page and in diagnostics).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    pub os: &'static str,
    pub arch: &'static str,
    pub family: &'static str,
    pub display_server: String,
    pub debug_build: bool,
}

impl PlatformInfo {
    pub fn detect() -> Self {
        let display_server = if cfg!(target_os = "linux") {
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                "Wayland"
            } else if std::env::var_os("DISPLAY").is_some() {
                "X11"
            } else {
                "none detected"
            }
        } else if cfg!(target_os = "windows") {
            "Win32"
        } else if cfg!(target_os = "macos") {
            "Quartz"
        } else {
            "unknown"
        };
        Self {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            family: std::env::consts::FAMILY,
            display_server: display_server.to_string(),
            debug_build: cfg!(debug_assertions),
        }
    }
}

/// Resident set size of this process in bytes.
///
/// Implemented for Linux (`/proc/self/statm`). Other platforms return `None` rather than
/// pulling in a system-information dependency; see `docs/benchmarking.md`.
pub fn process_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
        let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
        Some(pages * 4096) // 4 KiB pages on every Linux target this project supports
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_fills_fields() {
        let i = PlatformInfo::detect();
        assert!(!i.os.is_empty() && !i.arch.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rss_is_reported_on_linux() {
        assert!(process_rss_bytes().unwrap() > 0);
    }
}
