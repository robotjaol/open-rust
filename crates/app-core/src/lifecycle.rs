//! Application lifecycle: bootstrap, panic strategy, crash reports and shutdown.

use crate::events::{EventBus, EventReceiver};
use crate::notify::NotificationCenter;
use crate::tasks::TaskManager;
use app_config::{Config, Environment};
use app_errors::{AppError, ErrorReport, Result, time};
use app_logging::{LogBuffer, LogOptions};
use app_platform::{AppPaths, PlatformInfo};
use app_security::secrets::{MemorySecretStore, SecretStore};
use app_storage::Database;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Written to `crashes/` by the panic hook; shown on the Recovery page at next start.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashReport {
    pub unix_time: u64,
    pub message: String,
    pub location: String,
    pub thread: String,
    pub backtrace: String,
    #[serde(skip)]
    pub file: PathBuf,
}

static HOOK_INSTALLED: OnceLock<()> = OnceLock::new();

/// Install a panic hook that logs the panic and writes a crash report, then defers to the
/// previous hook. Idempotent. The application keeps `panic = "unwind"` so task panics can be
/// contained; a panic on the UI thread ends the process, and the report is offered at next start.
pub fn install_panic_hook(crash_dir: PathBuf) {
    if HOOK_INSTALLED.set(()).is_err() {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic with non-text payload".into());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let thread = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_string();
        tracing::error!(%message, %location, %thread, "panic");
        // Task workers contain their own panics, so only persist reports for other threads.
        if !thread.starts_with("task-worker") {
            let report = CrashReport {
                unix_time: time::now_unix(),
                message,
                location,
                thread,
                backtrace: std::backtrace::Backtrace::force_capture().to_string(),
                file: PathBuf::new(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&report) {
                let _ = std::fs::create_dir_all(&crash_dir);
                let _ = std::fs::write(
                    crash_dir.join(format!("crash-{}.json", report.unix_time)),
                    json,
                );
            }
        }
        previous(info);
    }));
}

/// Most recent unacknowledged crash report, if any.
pub fn find_previous_crash(crash_dir: &Path) -> Option<CrashReport> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(crash_dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let file = files.pop()?;
    let mut report: CrashReport =
        serde_json::from_str(&std::fs::read_to_string(&file).ok()?).ok()?;
    report.file = file;
    Some(report)
}

/// Mark a crash report as handled by moving it to `crashes/archived/`.
pub fn acknowledge_crash(report: &CrashReport) -> Result<()> {
    let Some(dir) = report.file.parent() else {
        return Ok(());
    };
    let archive = dir.join("archived");
    std::fs::create_dir_all(&archive).map_err(|e| AppError::io(&archive, e))?;
    let Some(name) = report.file.file_name() else {
        return Ok(());
    };
    std::fs::rename(&report.file, archive.join(name)).map_err(|e| AppError::io(&report.file, e))
}

#[derive(Debug, Clone)]
pub struct BootOptions {
    /// Redirect all directories below this path (tests, screenshots, portable mode).
    pub home: Option<PathBuf>,
    pub environment: Option<Environment>,
    /// Install the global tracing subscriber and panic hook. Disable in unit tests.
    pub init_process_hooks: bool,
    pub seed_rows: usize,
    pub worker_threads: usize,
}

impl Default for BootOptions {
    fn default() -> Self {
        Self {
            home: None,
            environment: None,
            init_process_hooks: true,
            seed_rows: 240,
            worker_threads: 2,
        }
    }
}

/// Everything the UI needs, owned in one place and passed explicitly (no globals).
pub struct AppContext {
    pub environment: Environment,
    pub paths: AppPaths,
    pub platform: PlatformInfo,
    pub config: Config,
    /// Set when the settings file was unreadable and defaults were substituted.
    pub config_problem: Option<ErrorReport>,
    pub db: Database,
    pub tasks: TaskManager,
    pub events: EventBus,
    pub notifications: NotificationCenter,
    pub logs: LogBuffer,
    pub log_file: Option<PathBuf>,
    pub secrets: Arc<dyn SecretStore>,
    pub previous_crash: Option<CrashReport>,
    /// Wall-clock time spent in [`AppContext::boot`].
    pub boot_time: Duration,
}

impl std::fmt::Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContext")
            .field("environment", &self.environment)
            .field("boot_time", &self.boot_time)
            .finish_non_exhaustive()
    }
}

impl AppContext {
    /// Initialise configuration, logging, storage and background services.
    /// Returns the context and the receiver the UI polls for [`crate::AppEvent`]s.
    pub fn boot(opts: &BootOptions) -> Result<(Self, EventReceiver)> {
        let started = Instant::now();
        let environment = opts.environment.unwrap_or_else(Environment::detect);
        let paths = match &opts.home {
            Some(h) => AppPaths::under(h, true),
            None => AppPaths::discover()?,
        };
        paths.ensure_exist()?;

        let (mut config, config_error) = Config::load_or_default(&paths.config_file(), environment);
        config.apply_env_overrides();

        let (logs, log_file) = if opts.init_process_hooks {
            let handle = app_logging::init(&LogOptions {
                filter: config.logging.level.clone(),
                dir: config.logging.to_file.then(|| paths.log_dir.clone()),
                stderr: true,
                buffer_capacity: 2000,
            })?;
            install_panic_hook(paths.crash_dir.clone());
            (handle.buffer, handle.file)
        } else {
            (LogBuffer::new(2000), None)
        };

        tracing::info!(
            version = env!("CARGO_PKG_VERSION"),
            ?environment,
            "starting"
        );
        if let Some(e) = &config_error {
            tracing::warn!(error = %e, "settings problem; using defaults");
        }

        let db = Database::open(&paths.database_file())?;
        if opts.seed_rows > 0 {
            let n = db.seed_demo_data(opts.seed_rows)?;
            if n > 0 {
                tracing::info!(rows = n, "seeded demo data");
            }
        }

        let (events, rx) = EventBus::new();
        let tasks = TaskManager::new(opts.worker_threads, events.clone());
        let previous_crash = find_previous_crash(&paths.crash_dir);
        if previous_crash.is_some() {
            tracing::warn!("a crash report from the previous session was found");
        }

        let ctx = Self {
            environment,
            platform: PlatformInfo::detect(),
            config_problem: config_error.map(|e| e.report()),
            config,
            db,
            tasks,
            events,
            notifications: NotificationCenter::default(),
            logs,
            log_file,
            secrets: Arc::new(MemorySecretStore::new()),
            previous_crash,
            paths,
            boot_time: started.elapsed(),
        };
        tracing::info!(boot_ms = ctx.boot_time.as_millis() as u64, "boot complete");
        Ok((ctx, rx))
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save(&self.paths.config_file())
    }

    /// Flush state and stop background work. Safe to call more than once.
    pub fn shutdown(&mut self) {
        tracing::info!("shutting down");
        if let Err(e) = self.save_config() {
            tracing::error!(error = %e, "could not save settings on exit");
        }
        let clean = self.tasks.shutdown(Duration::from_secs(2));
        if !clean {
            tracing::warn!("some tasks did not stop within the grace period");
        }
        if let Err(e) = self.db.checkpoint() {
            tracing::error!(error = %e, "database checkpoint failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(dir: &Path) -> BootOptions {
        BootOptions {
            home: Some(dir.to_path_buf()),
            environment: Some(Environment::Production),
            init_process_hooks: false,
            seed_rows: 30,
            worker_threads: 1,
        }
    }

    #[test]
    fn boot_creates_directories_database_and_demo_data() {
        let d = tempfile::tempdir().unwrap();
        let (ctx, _rx) = AppContext::boot(&opts(d.path())).unwrap();
        assert!(ctx.paths.database_file().exists());
        assert_eq!(ctx.db.count_items().unwrap(), 30);
        assert!(ctx.previous_crash.is_none());
    }

    #[test]
    fn config_persists_across_boots() {
        let d = tempfile::tempdir().unwrap();
        {
            let (mut ctx, _rx) = AppContext::boot(&opts(d.path())).unwrap();
            ctx.config.appearance.accent = app_config::Accent::Teal;
            ctx.shutdown();
        }
        let (ctx, _rx) = AppContext::boot(&opts(d.path())).unwrap();
        assert_eq!(ctx.config.appearance.accent, app_config::Accent::Teal);
    }

    #[test]
    fn corrupt_config_boots_with_defaults_and_reports_problem() {
        let d = tempfile::tempdir().unwrap();
        let paths = AppPaths::under(d.path(), true);
        paths.ensure_exist().unwrap();
        std::fs::write(paths.config_file(), "][ not toml").unwrap();
        let (ctx, _rx) = AppContext::boot(&opts(d.path())).unwrap();
        assert!(ctx.config_problem.is_some());
        assert_eq!(ctx.config.appearance, app_config::Appearance::default());
    }

    #[test]
    fn shutdown_is_idempotent() {
        let d = tempfile::tempdir().unwrap();
        let (mut ctx, _rx) = AppContext::boot(&opts(d.path())).unwrap();
        ctx.shutdown();
        ctx.shutdown();
    }

    #[test]
    fn crash_reports_are_found_and_acknowledged() {
        let d = tempfile::tempdir().unwrap();
        let crash_dir = d.path().join("crashes");
        std::fs::create_dir_all(&crash_dir).unwrap();
        let r = CrashReport {
            unix_time: 1,
            message: "boom".into(),
            location: "x.rs:1".into(),
            thread: "main".into(),
            backtrace: String::new(),
            file: PathBuf::new(),
        };
        std::fs::write(
            crash_dir.join("crash-1.json"),
            serde_json::to_string(&r).unwrap(),
        )
        .unwrap();
        let found = find_previous_crash(&crash_dir).unwrap();
        assert_eq!(found.message, "boom");
        acknowledge_crash(&found).unwrap();
        assert!(find_previous_crash(&crash_dir).is_none());
        assert!(crash_dir.join("archived/crash-1.json").exists());
    }

    #[test]
    fn boot_reports_previous_crash() {
        let d = tempfile::tempdir().unwrap();
        let paths = AppPaths::under(d.path(), true);
        paths.ensure_exist().unwrap();
        std::fs::write(
            paths.crash_dir.join("crash-5.json"),
            r#"{"unix_time":5,"message":"m","location":"l","thread":"t","backtrace":""}"#,
        )
        .unwrap();
        let (ctx, _rx) = AppContext::boot(&opts(d.path())).unwrap();
        assert_eq!(ctx.previous_crash.unwrap().unix_time, 5);
    }
}
