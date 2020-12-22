use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use triads::{arc_consistency::ac_3, arc_consistency::arc_consistency, core_triads::Triad};

fn ac_benchmark(c: &mut Criterion) {
    let triad = Triad::from_strings(
        "0100100001001000010010000100100001001000010010000100100001001000",
        "1110011111100111111001111110011111100111111001111110011111100111",
        "1110111011101110111011101110111011101110111011101110111011101110",
    );
    let list = triad.adjacency_list();

    c.bench_function("ac-3", |b| {
        b.iter(|| ac_3(black_box(&list), black_box(&list), HashMap::new()))
    });
}

fn ac3_benchmark(c: &mut Criterion) {
    let triad = Triad::from_strings(
        "0100100001001000010010000100100001001000010010000100100001001000",
        "1110011111100111111001111110011111100111111001111110011111100111",
        "1110111011101110111011101110111011101110111011101110111011101110",
    );
    let list = triad.adjacency_list();

    c.bench_function("arc_consistency", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

criterion_group!(benches, ac_benchmark, ac3_benchmark);
criterion_main!(benches);
