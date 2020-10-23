extern crate nalgebra_glm as glm;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use svo::fluid;

fn criterion_benchmark(c: &mut Criterion) {
    let mut fluidsim = fluid::Simulator::waterbox();
    c.bench_function("fluid simulation", |b| {
        b.iter(|| black_box(fluidsim.step()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
