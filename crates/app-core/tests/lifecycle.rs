#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! End-to-end tests of the headless application core: boot, storage, tasks, backup.

use app_core::{AppContext, AppEvent, BootOptions, TaskStatus};
use std::time::{Duration, Instant};

fn boot(dir: &std::path::Path) -> (AppContext, app_core::EventReceiver) {
    let opts = BootOptions {
        home: Some(dir.to_path_buf()),
        init_process_hooks: false,
        seed_rows: 25,
        worker_threads: 2,
        ..BootOptions::default()
    };
    AppContext::boot(&opts).expect("boot")
}

fn wait_finished(ctx: &AppContext, id: u64) -> app_core::TaskInfo {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let info = ctx.tasks.get(id).expect("task exists");
        if info.status.is_finished() || Instant::now() > end {
            return info;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn boot_creates_database_and_seed_data() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, _rx) = boot(dir.path());
    assert_eq!(ctx.db.count_items().unwrap(), 25);
    assert!(ctx.db.schema_version().unwrap() >= 2);
    assert!(ctx.paths.database_file().exists());
}

#[test]
fn second_boot_keeps_data_and_does_not_reseed() {
    let dir = tempfile::tempdir().unwrap();
    {
        let (mut ctx, _rx) = boot(dir.path());
        ctx.shutdown();
    }
    let (ctx, _rx) = boot(dir.path());
    assert_eq!(ctx.db.count_items().unwrap(), 25);
}

#[test]
fn task_failure_and_panic_are_isolated() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, _rx) = boot(dir.path());
    let bad = ctx.tasks.spawn("fails", |_| {
        Err(app_errors::AppError::Validation("nope".into()))
    });
    let boom = ctx
        .tasks
        .spawn("panics", |_| -> app_errors::Result<String> {
            panic!("expected test panic")
        });
    let ok = ctx.tasks.spawn("works", |_| Ok("fine".into()));
    assert_eq!(wait_finished(&ctx, bad).status, TaskStatus::Failed);
    assert_eq!(wait_finished(&ctx, boom).status, TaskStatus::Failed);
    let done = wait_finished(&ctx, ok);
    assert_eq!(
        done.status,
        TaskStatus::Succeeded,
        "pool survives a panicking job"
    );
}

#[test]
fn running_task_can_be_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, _rx) = boot(dir.path());
    let id = ctx.tasks.spawn("loop", |t| {
        for _ in 0..1000 {
            t.check_cancelled()?;
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok("finished".into())
    });
    std::thread::sleep(Duration::from_millis(40));
    ctx.tasks.cancel(id);
    assert_eq!(wait_finished(&ctx, id).status, TaskStatus::Cancelled);
}

#[test]
fn backup_then_restore_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, _rx) = boot(dir.path());
    let backup = dir.path().join("backup.sqlite3");
    ctx.db.backup_to(&backup).unwrap();
    let before = ctx.db.count_items().unwrap();
    ctx.db
        .insert_item(&app_storage::NewItem {
            name: "extra".into(),
            category: "Test".into(),
            status: "active".into(),
            amount: 1.0,
        })
        .unwrap();
    assert_eq!(ctx.db.count_items().unwrap(), before + 1);
    ctx.db.restore_from(&backup).unwrap();
    assert_eq!(ctx.db.count_items().unwrap(), before);
}

#[test]
fn restore_rejects_a_file_that_is_not_a_database() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, _rx) = boot(dir.path());
    let junk = dir.path().join("junk.sqlite3");
    std::fs::write(&junk, b"this is not sqlite").unwrap();
    assert!(ctx.db.restore_from(&junk).is_err());
    assert_eq!(ctx.db.count_items().unwrap(), 25, "live database untouched");
}

#[test]
fn notifications_arrive_through_the_event_bus() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, rx) = boot(dir.path());
    ctx.events
        .notify(app_core::Notification::info("hello", "world"));
    let got = rx.drain();
    assert!(
        got.iter()
            .any(|e| matches!(e, AppEvent::Notify(n) if n.title == "hello"))
    );
}

#[test]
fn corrupt_config_falls_back_to_defaults_and_keeps_the_file() {
    let dir = tempfile::tempdir().unwrap();
    {
        let (mut ctx, _rx) = boot(dir.path());
        ctx.shutdown();
    }
    let cfg = {
        let (ctx, _rx) = boot(dir.path());
        ctx.paths.config_file()
    };
    std::fs::write(&cfg, "this = is [not valid toml").unwrap();
    let (ctx, _rx) = boot(dir.path());
    assert!(ctx.config_problem.is_some());
    let kept = std::fs::read_dir(cfg.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .any(|e| e.file_name().to_string_lossy().contains("corrupt"));
    assert!(kept, "unreadable settings are preserved for the user");
}
