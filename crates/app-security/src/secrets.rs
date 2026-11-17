//! Secret storage. Credentials must live in the operating system's credential facility
//! (Windows Credential Manager, macOS Keychain, Secret Service on Linux), never in config files.
//!
//! This crate defines the trait and an in-memory implementation for tests and previews.
//! A production OS-backed implementation is documented in `docs/security.md`
//! and is intentionally an opt-in integration so the core stays free of system libraries.

use app_errors::{AppError, Result};
use std::collections::HashMap;
use std::sync::Mutex;

/// Minimal secret store interface.
pub trait SecretStore: Send + Sync {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<()>;
    fn get(&self, service: &str, account: &str) -> Result<Option<String>>;
    fn delete(&self, service: &str, account: &str) -> Result<()>;
    /// Human-readable backend name, e.g. "memory".
    fn backend(&self) -> &'static str;
}

/// Process-local store. Secrets vanish on exit: fine for tests and demos, wrong for production.
#[derive(Debug, Default)]
pub struct MemorySecretStore {
    inner: Mutex<HashMap<(String, String), String>>,
}

impl MemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, HashMap<(String, String), String>>> {
        self.inner
            .lock()
            .map_err(|_| AppError::Internal("secret store lock poisoned".into()))
    }
}

impl SecretStore for MemorySecretStore {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<()> {
        self.lock()?
            .insert((service.into(), account.into()), secret.into());
        Ok(())
    }

    fn get(&self, service: &str, account: &str) -> Result<Option<String>> {
        Ok(self
            .lock()?
            .get(&(service.to_string(), account.to_string()))
            .cloned())
    }

    fn delete(&self, service: &str, account: &str) -> Result<()> {
        self.lock()?
            .remove(&(service.to_string(), account.to_string()));
        Ok(())
    }

    fn backend(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_delete() {
        let s = MemorySecretStore::new();
        assert_eq!(s.get("svc", "me").unwrap(), None);
        s.set("svc", "me", "token").unwrap();
        assert_eq!(s.get("svc", "me").unwrap().as_deref(), Some("token"));
        s.delete("svc", "me").unwrap();
        assert_eq!(s.get("svc", "me").unwrap(), None);
    }
}
