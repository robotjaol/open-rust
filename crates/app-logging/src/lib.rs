//! Structured logging built on `tracing`.
//!
//! * human-readable output on stderr,
//! * an optional append-only log file with size-based rotation at start-up,
//! * an in-memory ring buffer so the UI can show live logs without parsing files.

use app_errors::{AppError, Result, time};
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Maximum size of the active log file before it is rotated on start-up.
const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const ALL: [LogLevel; 5] = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl From<&Level> for LogLevel {
    fn from(l: &Level) -> Self {
        match *l {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

/// One captured log line.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub seq: u64,
    pub unix_time: u64,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

#[derive(Debug)]
struct BufferInner {
    records: VecDeque<LogRecord>,
    capacity: usize,
    next_seq: u64,
}

/// Cheaply clonable, thread-safe ring buffer of recent log records.
#[derive(Debug, Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<BufferInner>>,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BufferInner {
                records: VecDeque::with_capacity(capacity.min(4096)),
                capacity: capacity.max(1),
                next_seq: 0,
            })),
        }
    }

    pub fn push(&self, level: LogLevel, target: &str, message: String) {
        // A poisoned lock must never take the application down just because a log call ran.
        let Ok(mut g) = self.inner.lock() else { return };
        let seq = g.next_seq;
        g.next_seq += 1;
        if g.records.len() == g.capacity {
            g.records.pop_front();
        }
        g.records.push_back(LogRecord {
            seq,
            unix_time: time::now_unix(),
            level,
            target: target.to_string(),
            message,
        });
    }

    /// Snapshot of all buffered records, oldest first.
    pub fn snapshot(&self) -> Vec<LogRecord> {
        self.inner
            .lock()
            .map(|g| g.records.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().map(|g| g.records.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.inner.lock() {
            g.records.clear();
        }
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
    fields: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else {
            let _ = write!(self.fields, " {}={value:?}", field.name());
        }
    }
}

struct RingLayer {
    buffer: LogBuffer,
}

impl<S: Subscriber> Layer<S> for RingLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut v = MessageVisitor::default();
        event.record(&mut v);
        let mut text = v.message;
        text.push_str(&v.fields);
        self.buffer.push(
            LogLevel::from(event.metadata().level()),
            event.metadata().target(),
            text,
        );
    }
}

/// Logging set-up options.
#[derive(Debug, Clone)]
pub struct LogOptions {
    /// `EnvFilter` directive, e.g. `info` or `open_rust=debug,warn`. `RUST_LOG` wins if set.
    pub filter: String,
    /// Directory for `open-rust.log`. `None` disables file logging.
    pub dir: Option<PathBuf>,
    /// Write human-readable lines to stderr.
    pub stderr: bool,
    pub buffer_capacity: usize,
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            filter: "info".into(),
            dir: None,
            stderr: true,
            buffer_capacity: 2000,
        }
    }
}

/// Returned by [`init`]; keeps the live buffer and the log file location.
#[derive(Debug, Clone)]
pub struct LogHandle {
    pub buffer: LogBuffer,
    pub file: Option<PathBuf>,
}

fn rotate_if_large(path: &Path) {
    if std::fs::metadata(path)
        .map(|m| m.len() > MAX_LOG_BYTES)
        .unwrap_or(false)
    {
        let _ = std::fs::rename(path, path.with_extension("log.1"));
    }
}

/// Install the global subscriber. Safe to call once per process; later calls return an error.
pub fn init(opts: &LogOptions) -> Result<LogHandle> {
    let buffer = LogBuffer::new(opts.buffer_capacity);
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&opts.filter))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let mut file_path = None;
    let file_layer = match &opts.dir {
        Some(dir) => {
            std::fs::create_dir_all(dir).map_err(|e| AppError::io(dir, e))?;
            let path = dir.join("open-rust.log");
            rotate_if_large(&path);
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| AppError::io(&path, e))?;
            file_path = Some(path);
            Some(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_writer(Mutex::new(file)),
            )
        }
        None => None,
    };
    let stderr_layer = opts
        .stderr
        .then(|| tracing_subscriber::fmt::layer().with_writer(std::io::stderr));

    tracing_subscriber::registry()
        .with(filter)
        .with(RingLayer {
            buffer: buffer.clone(),
        })
        .with(file_layer)
        .with(stderr_layer)
        .try_init()
        .map_err(|e| AppError::Internal(format!("logging already initialised: {e}")))?;

    Ok(LogHandle {
        buffer,
        file: file_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_drops_oldest() {
        let b = LogBuffer::new(3);
        for i in 0..5 {
            b.push(LogLevel::Info, "t", format!("m{i}"));
        }
        let s = b.snapshot();
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].message, "m2");
        assert_eq!(s[2].seq, 4);
    }

    #[test]
    fn init_captures_events_and_writes_file() {
        let dir = tempfile::tempdir().unwrap();
        let handle = init(&LogOptions {
            filter: "debug".into(),
            dir: Some(dir.path().to_path_buf()),
            stderr: false,
            buffer_capacity: 10,
        })
        .unwrap();
        tracing::info!(answer = 42, "hello logging");
        let recs = handle.buffer.snapshot();
        assert!(
            recs.iter()
                .any(|r| r.message.contains("hello logging") && r.message.contains("answer=42"))
        );
        let text = std::fs::read_to_string(handle.file.unwrap()).unwrap();
        assert!(text.contains("hello logging"));
        // A second init in the same process must fail cleanly rather than panic.
        assert!(init(&LogOptions::default()).is_err());
    }
}
