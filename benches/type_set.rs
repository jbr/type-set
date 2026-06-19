use criterion::{black_box, criterion_group, criterion_main, Criterion};
use type_set::TypeSet;

// A handful of distinct, cheap-to-construct types to populate a set with.
fn fill(set: &mut TypeSet) {
    set.insert(1u8);
    set.insert(2u32);
    set.insert(3u64);
    set.insert("four");
}

fn benches(c: &mut Criterion) {
    // Constructing a fresh set, filling it, and dropping it -- the "allocate a type set per
    // request" pattern that motivated moving off `BTreeMap`.
    c.bench_function("new + fill + drop", |b| {
        b.iter(|| {
            let mut set = TypeSet::new();
            fill(&mut set);
            black_box(&set);
        });
    });

    // Same, but preallocating capacity up front.
    c.bench_function("with_capacity + fill + drop", |b| {
        b.iter(|| {
            let mut set = TypeSet::with_capacity(4);
            fill(&mut set);
            black_box(&set);
        });
    });

    // Reusing a single allocation across many fill cycles via `clear`. This is the path that
    // `BTreeMap` could not accelerate, because its `clear` frees the backing nodes.
    c.bench_function("clear + fill (reuse)", |b| {
        let mut set = TypeSet::with_capacity(4);
        b.iter(|| {
            set.clear();
            fill(&mut set);
            black_box(&set);
        });
    });

    // The read hot path: four lookups against a populated set.
    c.bench_function("4 gets", |b| {
        let mut set = TypeSet::with_capacity(4);
        fill(&mut set);
        b.iter(|| {
            black_box(set.get::<u8>());
            black_box(set.get::<u32>());
            black_box(set.get::<u64>());
            black_box(set.get::<&'static str>());
        });
    });
}

criterion_group!(group, benches);
criterion_main!(group);
