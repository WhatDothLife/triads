use criterion::{black_box, criterion_group, criterion_main, Criterion};
use triads::{arc_consistency::ac_3, arc_consistency::arc_consistency, core_triads::Triad};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn ac_benchmark(c: &mut Criterion) {
    let triad = Triad::from_strings(
        "0100111101101001111011",
        "01100000110110000011",
        "10100001001010000100",
    );
    let list = triad.adjacency_list();

    // c.bench_function("ac-3", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("ac-3", |b| {
        b.iter(|| ac_3(black_box(&list), black_box(&list)))
    });
}

fn ac3_benchmark(c: &mut Criterion) {
    let triad = Triad::from_strings(
        "0100111101101001111011",
        "01100000110110000011",
        "10100001001010000100",
    );
    let list = triad.adjacency_list();

    c.bench_function("arc_consistency", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

criterion_group!(benches, ac_benchmark, ac3_benchmark);
criterion_main!(benches);
