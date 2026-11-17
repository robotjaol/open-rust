//! Thread-safe event channel from background threads to the UI thread.

use crate::notify::Notification;
use app_errors::ErrorReport;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, OnceLock};

/// What a native dialog was opened for, so the UI can route the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogPurpose {
    ImportJson,
    ExportCsv,
    ExportJson,
    BackupDatabase,
    RestoreDatabase,
    BrowseFolder,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Notify(Notification),
    /// A task changed state or progress. The UI re-reads the task table.
    TaskChanged,
    DialogResult {
        purpose: DialogPurpose,
        path: Option<PathBuf>,
    },
    Error(ErrorReport),
}

type Waker = Box<dyn Fn() + Send + Sync>;

/// Sender side. Cheap to clone; hand one to every background component.
#[derive(Clone)]
pub struct EventBus {
    tx: Sender<AppEvent>,
    waker: Arc<OnceLock<Waker>>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus").finish_non_exhaustive()
    }
}

/// Receiver side, owned by the UI thread.
#[derive(Debug)]
pub struct EventReceiver {
    rx: Receiver<AppEvent>,
}

impl EventBus {
    pub fn new() -> (Self, EventReceiver) {
        let (tx, rx) = channel();
        (
            Self {
                tx,
                waker: Arc::new(OnceLock::new()),
            },
            EventReceiver { rx },
        )
    }

    /// Register a callback that wakes the UI (e.g. `ctx.request_repaint`). First call wins.
    pub fn set_waker(&self, f: impl Fn() + Send + Sync + 'static) {
        let _ = self.waker.set(Box::new(f));
    }

    /// Send an event. A closed receiver (application shutting down) is not an error.
    pub fn send(&self, event: AppEvent) {
        let _ = self.tx.send(event);
        if let Some(w) = self.waker.get() {
            w();
        }
    }

    pub fn notify(&self, n: Notification) {
        self.send(AppEvent::Notify(n));
    }
}

impl EventReceiver {
    /// Drain everything currently queued without blocking.
    pub fn drain(&self) -> Vec<AppEvent> {
        self.rx.try_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn events_arrive_in_order_and_wake_the_ui() {
        let (bus, rx) = EventBus::new();
        let wakes = Arc::new(AtomicUsize::new(0));
        let w = wakes.clone();
        bus.set_waker(move || {
            w.fetch_add(1, Ordering::SeqCst);
        });
        bus.send(AppEvent::TaskChanged);
        bus.send(AppEvent::DialogResult {
            purpose: DialogPurpose::ImportJson,
            path: None,
        });
        let got = rx.drain();
        assert_eq!(got.len(), 2);
        assert!(matches!(got[0], AppEvent::TaskChanged));
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
        assert!(rx.drain().is_empty());
    }

    #[test]
    fn sending_after_receiver_dropped_is_harmless() {
        let (bus, rx) = EventBus::new();
        drop(rx);
        bus.send(AppEvent::TaskChanged);
    }

    #[test]
    fn bus_can_be_used_from_threads() {
        let (bus, rx) = EventBus::new();
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let b = bus.clone();
                std::thread::spawn(move || b.send(AppEvent::TaskChanged))
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
        assert_eq!(rx.drain().len(), 4);
    }
}
