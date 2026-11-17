#![allow(clippy::unwrap_used, clippy::expect_used)]
use app_storage::{Database, ItemQuery, ItemSort};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_storage(c: &mut Criterion) {
    let db = Database::open_in_memory().unwrap();
    db.seed_demo_data(10_000).unwrap();

    c.bench_function("query_items: page of 50, sorted by name, 10k rows", |b| {
        let q = ItemQuery {
            sort: ItemSort::Name,
            limit: 50,
            ..ItemQuery::default()
        };
        b.iter(|| black_box(db.query_items(black_box(&q)).unwrap()));
    });
    c.bench_function("query_items: text search, 10k rows", |b| {
        let q = ItemQuery {
            search: "Archive 05".into(),
            limit: 50,
            ..ItemQuery::default()
        };
        b.iter(|| black_box(db.query_items(black_box(&q)).unwrap()));
    });
    c.bench_function("item_stats: aggregate, 10k rows", |b| {
        b.iter(|| black_box(db.item_stats().unwrap()));
    });
    c.bench_function("export_items_csv: 10k rows", |b| {
        b.iter(|| black_box(db.export_items_csv().unwrap()));
    });
}

criterion_group!(benches, bench_storage);
criterion_main!(benches);
