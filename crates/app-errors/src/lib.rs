//! Typed application errors with user-safe messages.
//!
//! Internal details (paths, SQL, OS messages) are kept in [`AppError`] for logs.
//! UI code must only ever show [`AppError::user_message`] or [`ErrorReport::user_message`].

pub mod time;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

/// Convenient result alias used by every application crate.
pub type Result<T, E = AppError> = std::result::Result<T, E>;

/// Broad error category, stable enough to be matched on by UI code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorKind {
    Config,
    Storage,
    Security,
    Io,
    Validation,
    Cancelled,
    Platform,
    Internal,
}

/// The application-wide error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("security policy violation: {0}")]
    Security(String),
    #[error("i/o error at `{path}`: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("operation cancelled")]
    Cancelled,
    #[error("platform error: {0}")]
    Platform(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Wrap an [`std::io::Error`] together with the path it relates to.
    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().display().to_string(),
            source,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Config(_) => ErrorKind::Config,
            Self::Storage(_) => ErrorKind::Storage,
            Self::Security(_) => ErrorKind::Security,
            Self::Io { .. } => ErrorKind::Io,
            Self::Validation(_) => ErrorKind::Validation,
            Self::Cancelled => ErrorKind::Cancelled,
            Self::Platform(_) => ErrorKind::Platform,
            Self::Internal(_) => ErrorKind::Internal,
        }
    }

    /// Stable machine-readable code, safe to show to users and to quote in bug reports.
    pub fn code(&self) -> &'static str {
        match self.kind() {
            ErrorKind::Config => "E-CONFIG",
            ErrorKind::Storage => "E-STORAGE",
            ErrorKind::Security => "E-SECURITY",
            ErrorKind::Io => "E-IO",
            ErrorKind::Validation => "E-VALIDATION",
            ErrorKind::Cancelled => "E-CANCELLED",
            ErrorKind::Platform => "E-PLATFORM",
            ErrorKind::Internal => "E-INTERNAL",
        }
    }

    /// A message that is safe to display. It never contains paths, SQL or OS error text.
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(_) => "The settings could not be read. Defaults were used instead.".into(),
            Self::Storage(_) => {
                "The local database reported a problem. Your data was not changed.".into()
            }
            Self::Security(_) => {
                "The action was blocked by the application's security policy.".into()
            }
            Self::Io { path, source } => {
                let name = Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "the requested file".into());
                let why = match source.kind() {
                    std::io::ErrorKind::NotFound => "it does not exist",
                    std::io::ErrorKind::PermissionDenied => "permission was denied",
                    std::io::ErrorKind::AlreadyExists => "it already exists",
                    _ => "an input/output problem occurred",
                };
                format!("Could not access `{name}` because {why}.")
            }
            // Validation messages are authored by this application and are safe by contract.
            Self::Validation(msg) => msg.clone(),
            Self::Cancelled => "The operation was cancelled.".into(),
            Self::Platform(_) => {
                "A system integration feature is unavailable on this device.".into()
            }
            Self::Internal(_) => {
                "Something unexpected happened. The details were saved to the log.".into()
            }
        }
    }

    /// Whether the application can keep running normally after this error.
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, Self::Internal(_))
    }

    /// Build a serialisable report. Internal text goes to `details`, which is log-only.
    pub fn report(&self) -> ErrorReport {
        ErrorReport {
            code: self.code().to_string(),
            kind: self.kind(),
            user_message: self.user_message(),
            details: self.to_string(),
            recoverable: self.is_recoverable(),
            unix_time: time::now_unix(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(source: std::io::Error) -> Self {
        Self::Io {
            path: String::new(),
            source,
        }
    }
}

/// Serialisable error snapshot used by the error centre and crash reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorReport {
    pub code: String,
    pub kind: ErrorKind,
    pub user_message: String,
    /// Internal description. Log-only: never render this in the UI.
    pub details: String,
    pub recoverable: bool,
    pub unix_time: u64,
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.user_message)
    }
}

/// Extension trait that attaches context while converting into [`AppError`].
pub trait Context<T> {
    fn context(self, what: &str) -> Result<T>;
}

impl<T, E: std::error::Error> Context<T> for std::result::Result<T, E> {
    fn context(self, what: &str) -> Result<T> {
        self.map_err(|e| AppError::Internal(format!("{what}: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_message_hides_internal_details() {
        let e = AppError::Storage("near \"SELECT secret FROM x\": syntax error".into());
        assert!(!e.user_message().contains("SELECT"));
        assert!(e.to_string().contains("SELECT"));
    }

    #[test]
    fn io_message_shows_file_name_only() {
        let e = AppError::io(
            "/home/alice/private/report.txt",
            std::io::Error::from(std::io::ErrorKind::NotFound),
        );
        let m = e.user_message();
        assert!(m.contains("report.txt"));
        assert!(!m.contains("alice"));
    }

    #[test]
    fn report_round_trips_through_json_like_serde() {
        let r = AppError::Validation("Name must not be empty".into()).report();
        assert_eq!(r.code, "E-VALIDATION");
        assert!(r.recoverable);
    }

    #[test]
    fn internal_errors_are_not_recoverable() {
        assert!(!AppError::Internal("x".into()).is_recoverable());
    }
}
