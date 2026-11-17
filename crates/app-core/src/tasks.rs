//! Background task infrastructure: a small fixed-size worker pool with progress reporting,
//! cooperative cancellation, panic isolation and graceful shutdown.
//!
//! Tasks are plain closures. They never touch the UI; they report through [`TaskContext`],
//! and the UI reads snapshots via [`TaskManager::snapshot`].

use crate::events::{AppEvent, EventBus};
use crate::notify::Notification;
use app_errors::{AppError, Result, time::now_unix};
use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub type TaskId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Succeeded => "Succeeded",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Read-only view of a task for the UI.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: TaskId,
    pub name: String,
    pub status: TaskStatus,
    /// 0.0 ..= 1.0
    pub progress: f32,
    pub message: String,
    pub created_unix: u64,
    pub finished_unix: Option<u64>,
    /// Outcome text for finished tasks (success summary or user-safe error).
    pub outcome: Option<String>,
}

/// Cheap, clonable cancellation flag.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Default)]
struct Table {
    infos: Vec<TaskInfo>,
    tokens: HashMap<TaskId, CancelToken>,
    next_id: TaskId,
}

type Job = Box<dyn FnOnce(&TaskContext) -> Result<String> + Send + 'static>;

struct Queue {
    jobs: VecDeque<(TaskId, Job)>,
    shutdown: bool,
}

struct Shared {
    table: Mutex<Table>,
    queue: Mutex<Queue>,
    cv: Condvar,
    bus: EventBus,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Tasks catch panics, so poisoning is unexpected; recover the data rather than cascade.
    m.lock().unwrap_or_else(|p| p.into_inner())
}

/// Handle passed to every task closure.
pub struct TaskContext {
    id: TaskId,
    token: CancelToken,
    shared: Arc<Shared>,
}

impl std::fmt::Debug for TaskContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskContext")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl TaskContext {
    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Returns `Err(AppError::Cancelled)` when cancellation was requested. Call it in loops
    /// and propagate with `?`.
    pub fn check_cancelled(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(AppError::Cancelled)
        } else {
            Ok(())
        }
    }

    /// Report progress (`fraction` is clamped to 0..=1).
    pub fn progress(&self, fraction: f32, message: impl Into<String>) {
        {
            let mut t = lock(&self.shared.table);
            if let Some(info) = t.infos.iter_mut().find(|i| i.id == self.id) {
                info.progress = if fraction.is_finite() {
                    fraction.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                info.message = message.into();
            }
        }
        self.shared.bus.send(AppEvent::TaskChanged);
    }
}

/// Worker pool and task registry.
pub struct TaskManager {
    shared: Arc<Shared>,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

impl std::fmt::Debug for TaskManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskManager").finish_non_exhaustive()
    }
}

const MAX_HISTORY: usize = 200;

impl TaskManager {
    /// Start `workers` threads (at least 1).
    pub fn new(workers: usize, bus: EventBus) -> Self {
        let shared = Arc::new(Shared {
            table: Mutex::new(Table::default()),
            queue: Mutex::new(Queue {
                jobs: VecDeque::new(),
                shutdown: false,
            }),
            cv: Condvar::new(),
            bus,
        });
        let handles = (0..workers.max(1))
            .filter_map(|n| {
                let s = shared.clone();
                std::thread::Builder::new()
                    .name(format!("task-worker-{n}"))
                    .spawn(move || worker_loop(s))
                    .map_err(|e| tracing::error!(error = %e, "could not start task worker"))
                    .ok()
            })
            .collect();
        Self {
            shared,
            workers: Mutex::new(handles),
        }
    }

    /// Queue a task. Returns its id immediately.
    pub fn spawn(
        &self,
        name: impl Into<String>,
        job: impl FnOnce(&TaskContext) -> Result<String> + Send + 'static,
    ) -> TaskId {
        let name = name.into();
        let id = {
            let mut t = lock(&self.shared.table);
            t.next_id += 1;
            let id = t.next_id;
            t.tokens.insert(id, CancelToken::default());
            t.infos.push(TaskInfo {
                id,
                name: name.clone(),
                status: TaskStatus::Queued,
                progress: 0.0,
                message: "Waiting for a worker".into(),
                created_unix: now_unix(),
                finished_unix: None,
                outcome: None,
            });
            id
        };
        tracing::info!(task = id, %name, "task queued");
        lock(&self.shared.queue).jobs.push_back((id, Box::new(job)));
        self.shared.cv.notify_one();
        self.shared.bus.send(AppEvent::TaskChanged);
        id
    }

    pub fn snapshot(&self) -> Vec<TaskInfo> {
        lock(&self.shared.table).infos.clone()
    }

    pub fn get(&self, id: TaskId) -> Option<TaskInfo> {
        lock(&self.shared.table)
            .infos
            .iter()
            .find(|i| i.id == id)
            .cloned()
    }

    /// Request cancellation. Queued tasks are cancelled immediately; running tasks stop at
    /// their next `check_cancelled`.
    pub fn cancel(&self, id: TaskId) {
        let token = lock(&self.shared.table).tokens.get(&id).cloned();
        if let Some(t) = token {
            t.cancel();
        }
        // Remove from the queue if it has not started.
        let was_queued = {
            let mut q = lock(&self.shared.queue);
            let before = q.jobs.len();
            q.jobs.retain(|(jid, _)| *jid != id);
            q.jobs.len() != before
        };
        if was_queued {
            finish(
                &self.shared,
                id,
                TaskStatus::Cancelled,
                "Cancelled before it started".into(),
            );
        }
        self.shared.bus.send(AppEvent::TaskChanged);
    }

    /// Drop finished tasks from the table.
    pub fn clear_finished(&self) {
        let mut t = lock(&self.shared.table);
        let done: Vec<TaskId> = t
            .infos
            .iter()
            .filter(|i| i.status.is_finished())
            .map(|i| i.id)
            .collect();
        t.infos.retain(|i| !i.status.is_finished());
        for id in done {
            t.tokens.remove(&id);
        }
        drop(t);
        self.shared.bus.send(AppEvent::TaskChanged);
    }

    pub fn running_count(&self) -> usize {
        lock(&self.shared.table)
            .infos
            .iter()
            .filter(|i| i.status == TaskStatus::Running)
            .count()
    }

    /// Cancel everything and wait up to `timeout` for workers to exit.
    /// Returns `true` if all workers stopped in time.
    pub fn shutdown(&self, timeout: Duration) -> bool {
        {
            let mut q = lock(&self.shared.queue);
            q.shutdown = true;
            q.jobs.clear();
        }
        for t in lock(&self.shared.table).tokens.values() {
            t.cancel();
        }
        self.shared.cv.notify_all();
        let deadline = Instant::now() + timeout;
        let handles: Vec<_> = lock(&self.workers).drain(..).collect();
        let mut clean = true;
        for h in handles {
            while !h.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
            if h.is_finished() {
                let _ = h.join();
            } else {
                clean = false; // detached: it holds no UI resources and exits with the process
            }
        }
        clean
    }
}

impl Drop for TaskManager {
    fn drop(&mut self) {
        self.shutdown(Duration::from_millis(500));
    }
}

fn finish(shared: &Shared, id: TaskId, status: TaskStatus, outcome: String) {
    let name = {
        let mut t = lock(&shared.table);
        let Some(info) = t.infos.iter_mut().find(|i| i.id == id) else {
            return;
        };
        info.status = status;
        info.finished_unix = Some(now_unix());
        if status == TaskStatus::Succeeded {
            info.progress = 1.0;
        }
        info.message = outcome.clone();
        info.outcome = Some(outcome.clone());
        let name = info.name.clone();
        // Keep the table bounded: drop the oldest finished entries.
        while t.infos.len() > MAX_HISTORY {
            if let Some(pos) = t.infos.iter().position(|i| i.status.is_finished()) {
                let removed = t.infos.remove(pos);
                t.tokens.remove(&removed.id);
            } else {
                break;
            }
        }
        name
    };
    let note = match status {
        TaskStatus::Succeeded => Some(Notification::success(format!("{name} finished"), outcome)),
        TaskStatus::Failed => Some(Notification::error(format!("{name} failed"), outcome)),
        TaskStatus::Cancelled => Some(Notification::warning(format!("{name} cancelled"), outcome)),
        _ => None,
    };
    if let Some(n) = note {
        shared.bus.notify(n);
    }
    shared.bus.send(AppEvent::TaskChanged);
}

fn worker_loop(shared: Arc<Shared>) {
    loop {
        let next = {
            let mut q = lock(&shared.queue);
            loop {
                if q.shutdown {
                    return;
                }
                if let Some(j) = q.jobs.pop_front() {
                    break j;
                }
                q = shared.cv.wait(q).unwrap_or_else(|p| p.into_inner());
            }
        };
        let (id, job) = next;
        let token = {
            let mut t = lock(&shared.table);
            let token = t.tokens.get(&id).cloned().unwrap_or_default();
            if let Some(info) = t.infos.iter_mut().find(|i| i.id == id) {
                info.status = TaskStatus::Running;
                info.message = "Running".into();
            }
            token
        };
        shared.bus.send(AppEvent::TaskChanged);
        let ctx = TaskContext {
            id,
            token,
            shared: shared.clone(),
        };
        match catch_unwind(AssertUnwindSafe(|| job(&ctx))) {
            Ok(Ok(summary)) => finish(&shared, id, TaskStatus::Succeeded, summary),
            Ok(Err(AppError::Cancelled)) => {
                finish(&shared, id, TaskStatus::Cancelled, "Cancelled".into())
            }
            Ok(Err(e)) => {
                tracing::warn!(task = id, error = %e, "task failed");
                finish(&shared, id, TaskStatus::Failed, e.user_message());
            }
            Err(_) => {
                tracing::error!(task = id, "task panicked");
                finish(
                    &shared,
                    id,
                    TaskStatus::Failed,
                    "The task stopped unexpectedly.".into(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventBus;

    fn wait_for(m: &TaskManager, id: TaskId, f: impl Fn(&TaskInfo) -> bool) -> TaskInfo {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(i) = m.get(id)
                && f(&i)
            {
                return i;
            }
            assert!(Instant::now() < end, "timed out waiting for task {id}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn manager(workers: usize) -> TaskManager {
        let (bus, rx) = EventBus::new();
        std::mem::forget(rx); // keep the channel open for the test's lifetime
        TaskManager::new(workers, bus)
    }

    #[test]
    fn successful_task_reports_outcome() {
        let m = manager(1);
        let id = m.spawn("ok", |ctx| {
            ctx.progress(0.5, "half");
            Ok("done".into())
        });
        let i = wait_for(&m, id, |i| i.status.is_finished());
        assert_eq!(i.status, TaskStatus::Succeeded);
        assert_eq!(i.outcome.as_deref(), Some("done"));
        assert_eq!(i.progress, 1.0);
    }

    #[test]
    fn failing_task_exposes_only_user_safe_text() {
        let m = manager(1);
        let id = m.spawn("bad", |_| {
            Err(AppError::Storage("SELECT * FROM secrets".into()))
        });
        let i = wait_for(&m, id, |i| i.status.is_finished());
        assert_eq!(i.status, TaskStatus::Failed);
        assert!(!i.outcome.unwrap().contains("SELECT"));
    }

    #[test]
    fn panicking_task_is_isolated_and_pool_survives() {
        let m = manager(1);
        let bad = m.spawn("panic", |_| panic!("kaboom"));
        assert_eq!(
            wait_for(&m, bad, |i| i.status.is_finished()).status,
            TaskStatus::Failed
        );
        let ok = m.spawn("after", |_| Ok("still alive".into()));
        assert_eq!(
            wait_for(&m, ok, |i| i.status.is_finished()).status,
            TaskStatus::Succeeded
        );
    }

    #[test]
    fn running_task_can_be_cancelled() {
        let m = manager(1);
        let id = m.spawn("loop", |ctx| {
            loop {
                ctx.check_cancelled()?;
                std::thread::sleep(Duration::from_millis(2));
            }
        });
        wait_for(&m, id, |i| i.status == TaskStatus::Running);
        m.cancel(id);
        assert_eq!(
            wait_for(&m, id, |i| i.status.is_finished()).status,
            TaskStatus::Cancelled
        );
    }

    #[test]
    fn queued_task_is_cancelled_without_running() {
        let m = manager(1);
        let gate = CancelToken::default();
        let g = gate.clone();
        let blocker = m.spawn("blocker", move |_| {
            while !g.is_cancelled() {
                std::thread::sleep(Duration::from_millis(2));
            }
            Ok("released".into())
        });
        wait_for(&m, blocker, |i| i.status == TaskStatus::Running);
        let ran = Arc::new(AtomicBool::new(false));
        let r = ran.clone();
        let queued = m.spawn("queued", move |_| {
            r.store(true, Ordering::SeqCst);
            Ok(String::new())
        });
        m.cancel(queued);
        assert_eq!(m.get(queued).unwrap().status, TaskStatus::Cancelled);
        gate.cancel();
        wait_for(&m, blocker, |i| i.status.is_finished());
        assert!(!ran.load(Ordering::SeqCst));
    }

    #[test]
    fn concurrency_is_limited_to_worker_count() {
        let m = manager(2);
        let live = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let peak = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let ids: Vec<_> = (0..6)
            .map(|n| {
                let (live, peak) = (live.clone(), peak.clone());
                m.spawn(format!("t{n}"), move |_| {
                    let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(15));
                    live.fetch_sub(1, Ordering::SeqCst);
                    Ok(String::new())
                })
            })
            .collect();
        for id in ids {
            wait_for(&m, id, |i| i.status.is_finished());
        }
        assert!(peak.load(Ordering::SeqCst) <= 2);
    }

    #[test]
    fn progress_is_clamped_and_clear_finished_works() {
        let m = manager(1);
        let id = m.spawn("p", |ctx| {
            ctx.progress(7.0, "over");
            ctx.progress(f32::NAN, "nan");
            Ok("x".into())
        });
        wait_for(&m, id, |i| i.status.is_finished());
        m.clear_finished();
        assert!(m.snapshot().is_empty());
    }

    #[test]
    fn shutdown_cancels_running_work_quickly() {
        let m = manager(2);
        let id = m.spawn("long", |ctx| {
            loop {
                ctx.check_cancelled()?;
                std::thread::sleep(Duration::from_millis(2));
            }
        });
        wait_for(&m, id, |i| i.status == TaskStatus::Running);
        let t = Instant::now();
        assert!(m.shutdown(Duration::from_secs(2)));
        assert!(t.elapsed() < Duration::from_secs(1));
    }
}
