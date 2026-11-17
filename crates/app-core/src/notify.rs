//! In-app notifications: toasts are derived from the same records that fill the history panel.

use app_errors::time::now_unix;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub level: Level,
    pub title: String,
    pub message: String,
    pub unix_time: u64,
}

impl Notification {
    pub fn new(level: Level, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: 0,
            level,
            title: title.into(),
            message: message.into(),
            unix_time: now_unix(),
        }
    }

    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Level::Info, title, message)
    }

    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Level::Success, title, message)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Level::Warning, title, message)
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Level::Error, title, message)
    }
}

const HISTORY_CAP: usize = 200;

/// Bounded history with an unread counter.
#[derive(Debug, Default)]
pub struct NotificationCenter {
    history: VecDeque<Notification>,
    next_id: u64,
    unread: usize,
}

impl NotificationCenter {
    /// Store a notification and return it with its assigned id (for toast display).
    pub fn push(&mut self, mut n: Notification) -> Notification {
        self.next_id += 1;
        n.id = self.next_id;
        if self.history.len() == HISTORY_CAP {
            self.history.pop_front();
        }
        self.history.push_back(n.clone());
        self.unread += 1;
        n
    }

    /// Newest first.
    pub fn history(&self) -> impl Iterator<Item = &Notification> {
        self.history.iter().rev()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    pub fn unread(&self) -> usize {
        self.unread
    }

    pub fn mark_all_read(&mut self) {
        self.unread = 0;
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.unread = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_history_is_newest_first() {
        let mut c = NotificationCenter::default();
        let a = c.push(Notification::info("a", ""));
        let b = c.push(Notification::error("b", ""));
        assert_ne!(a.id, b.id);
        assert_eq!(c.history().next().unwrap().title, "b");
        assert_eq!(c.unread(), 2);
        c.mark_all_read();
        assert_eq!(c.unread(), 0);
    }

    #[test]
    fn history_is_bounded() {
        let mut c = NotificationCenter::default();
        for i in 0..(HISTORY_CAP + 25) {
            c.push(Notification::info(format!("n{i}"), ""));
        }
        assert_eq!(c.len(), HISTORY_CAP);
        assert_eq!(
            c.history().next().unwrap().title,
            format!("n{}", HISTORY_CAP + 24)
        );
    }
}
