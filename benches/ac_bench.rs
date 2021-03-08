use criterion::{black_box, criterion_group, criterion_main, Criterion};
use triads::{arc_consistency::ac3, arc_consistency::arc_consistency, cores::Triad};

fn ac_triad_12(c: &mut Criterion) {
    let triad = Triad::from("0101", "1011", "111");
    let list = triad.adjacency_list();

    c.bench_function("ac_12", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

fn ac3_triad_12(c: &mut Criterion) {
    let triad = Triad::from("0101", "1011", "111");
    let list = triad.adjacency_list();

    c.bench_function("ac3_12", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn ac_triad_24(c: &mut Criterion) {
    let triad = Triad::from("01011101", "10111101", "1111011");
    let list = triad.adjacency_list();

    c.bench_function("ac_24", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

fn ac3_triad_24(c: &mut Criterion) {
    let triad = Triad::from("01011101", "10111101", "1111011");
    let list = triad.adjacency_list();

    c.bench_function("ac3_24", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn ac_triad_36(c: &mut Criterion) {
    let triad = Triad::from("010111011011", "101111010011", "11110110101");
    let list = triad.adjacency_list();

    c.bench_function("ac_36", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

fn ac3_triad_36(c: &mut Criterion) {
    let triad = Triad::from("010111011011", "101111010011", "11110110101");
    let list = triad.adjacency_list();

    c.bench_function("ac3_36", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn ac_triad_39(c: &mut Criterion) {
    let triad = Triad::from("0101110110111", "1011110100111", "111101101011");
    let list = triad.adjacency_list();

    c.bench_function("ac_36", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

fn ac3_triad_39(c: &mut Criterion) {
    let triad = Triad::from("0101110110111", "1011110100111", "111101101011");
    let list = triad.adjacency_list();

    c.bench_function("ac3_36", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn ac_triad_48(c: &mut Criterion) {
    let triad = Triad::from("010101011001011", "1011101010111011", "110000001110101");
    let list = triad.adjacency_list();

    c.bench_function("ac_48", |b| {
        b.iter(|| arc_consistency(black_box(&list), black_box(&list)))
    });
}

fn ac3_triad_48(c: &mut Criterion) {
    let triad = Triad::from("010101011001011", "1011101010111011", "110000001110101");
    let list = triad.adjacency_list();

    c.bench_function("ac3_48", |b| {
        b.iter(|| ac3(black_box(&list), black_box(&list)))
    });
}

fn commutative_ac_196(c: &mut Criterion) {
    let triad = Triad::from("01011", "10111", "1111");
    let list = triad.adjacency_list();
    let product = list.power(2);

    c.bench_function("commutative_ac_196", |b| {
        b.iter(|| arc_consistency(black_box(&product), black_box(&list)))
    });
}

fn commutative_ac3_196(c: &mut Criterion) {
    let triad = Triad::from("01011", "10111", "1111");
    let list = triad.adjacency_list();
    let product = list.power(2);

    c.bench_function("commutative_ac3_196", |b| {
        b.iter(|| ac3(black_box(&product), black_box(&list)))
    });
}

fn commutative_ac_361(c: &mut Criterion) {
    let triad = Triad::from("010111", "101111", "111101");
    let list = triad.adjacency_list();
    let product = list.power(2);

    let triad2 = Triad::from("01011", "10111", "1111");
    let list2 = triad2.adjacency_list();

    c.bench_function("commutative_ac_361", |b| {
        b.iter(|| arc_consistency(black_box(&product), black_box(&list2)))
    });
}

fn commutative_ac3_361(c: &mut Criterion) {
    let triad = Triad::from("010111", "101111", "111101");
    let list = triad.adjacency_list();
    let product = list.power(2);

    let triad2 = Triad::from("01011", "10111", "1111");
    let list2 = triad2.adjacency_list();

    c.bench_function("commutative_ac3_361", |b| {
        b.iter(|| ac3(black_box(&product), black_box(&list2)))
    });
}

fn commutative_ac_576(c: &mut Criterion) {
    let triad = Triad::from("01011101", "10111101", "1111011");
    let list = triad.adjacency_list();
    let product = list.power(2);

    let triad2 = Triad::from("01011", "10111", "1111");
    let list2 = triad2.adjacency_list();

    c.bench_function("commutative_ac_576", |b| {
        b.iter(|| arc_consistency(black_box(&product), black_box(&list2)))
    });
}

fn commutative_ac3_576(c: &mut Criterion) {
    let triad = Triad::from("01011101", "10111101", "1111011");
    let list = triad.adjacency_list();
    let product = list.power(2);

    let triad2 = Triad::from("01011", "10111", "1111");
    let list2 = triad2.adjacency_list();

    c.bench_function("commutative_ac3_576", |b| {
        b.iter(|| ac3(black_box(&product), black_box(&list2)))
    });
}
criterion_group!(
    ac,
    ac_triad_12,
    ac_triad_24,
    ac_triad_36,
    ac_triad_39,
    ac_triad_48
);
criterion_group!(
    ac_3,
    ac3_triad_12,
    ac3_triad_24,
    ac3_triad_36,
    ac3_triad_39,
    ac3_triad_48
);
criterion_group!(
    polymorphism_ac,
    commutative_ac_196,
    commutative_ac_361,
    commutative_ac_576
);
criterion_group!(
    polymorphism_ac3,
    commutative_ac3_196,
    commutative_ac3_361,
    commutative_ac3_576
);
criterion_main!(ac, ac_3);
