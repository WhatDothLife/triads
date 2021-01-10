use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use triads::{arc_consistency::ac3, arc_consistency::arc_consistency, core_triads::Triad};

fn ac3_benchmark(c: &mut Criterion) {
    let mut triad = Triad::new();
    triad.add_arm("01101011101001");
    // triad.add_arm("010110101110100110100");
    let list = triad.adjacency_list();

    c.bench_function("ac-3", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn ac3_benchmark_double(c: &mut Criterion) {
    let mut triad = Triad::new();
    triad.add_arm("0110101110100101001000101101");
    // triad.add_arm("010110101110100110100");
    let list = triad.adjacency_list();

    c.bench_function("ac_3_double", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn ac_benchmark(c: &mut Criterion) {
    let mut triad = Triad::new();
    triad.add_arm("01011010111010011010001001011110010110000000011");
    // triad.add_arm("010110101110100110100");
    let list = triad.adjacency_list();

    c.bench_function("arc_consistency", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

criterion_group!(benches, ac3_benchmark, ac3_benchmark_double);
criterion_main!(benches);
