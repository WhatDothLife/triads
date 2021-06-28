use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tripolys::{adjacency_list::AdjacencyList, consistency::sac_1, triad::Triad};

fn sac1_4(c: &mut Criterion) {
    let triad = Triad::from_str("0,0,0").unwrap();
    let list = AdjacencyList::<u32>::from(&triad);

    c.bench_function("sac1_4", |b| {
        b.iter(|| sac_1(black_box(&list), black_box(&list)))
    });
}

criterion_group!(sac_1, sac1_4);
criterion_main!(sac_1);
