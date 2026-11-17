#![allow(clippy::unwrap_used, clippy::expect_used)]
use app_core::{EventBus, TaskManager, TaskStatus};
use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

fn bench_tasks(c: &mut Criterion) {
    c.bench_function(
        "TaskManager: run 100 trivial tasks to completion (2 workers)",
        |b| {
            b.iter(|| {
                let (bus, _rx) = EventBus::new();
                let tm = TaskManager::new(2, bus);
                let ids: Vec<_> = (0..100)
                    .map(|i| tm.spawn(format!("t{i}"), |_| Ok(String::new())))
                    .collect();
                loop {
                    if ids.iter().all(|id| {
                        tm.get(*id)
                            .is_some_and(|t| t.status == TaskStatus::Succeeded)
                    }) {
                        break;
                    }
                    std::thread::yield_now();
                }
                tm.shutdown(Duration::from_secs(1));
            });
        },
    );
}

criterion_group!(benches, bench_tasks);
criterion_main!(benches);
