//! UI-agnostic application core.
//!
//! Nothing in this crate depends on a GUI toolkit, which keeps the business logic
//! testable and lets the front end be replaced.

pub mod events;
pub mod lifecycle;
pub mod notify;
pub mod state;
pub mod tasks;

pub use events::{AppEvent, DialogPurpose, EventBus, EventReceiver};
pub use lifecycle::{AppContext, BootOptions, CrashReport, acknowledge_crash};
pub use notify::{Level, Notification, NotificationCenter};
pub use state::{Action, AppState, NavGroup, Page};
pub use tasks::{CancelToken, TaskContext, TaskId, TaskInfo, TaskManager, TaskStatus};
