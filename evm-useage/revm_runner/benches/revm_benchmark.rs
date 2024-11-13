use criterion::{criterion_group, criterion_main, Criterion};
use revm_runner::run_revm_calc_contract;

fn evm_fibb_works() {
    let data = "61047ff4000000000000000000000000000000000000000000000000000000000000000a";
    let result = run_revm_calc_contract(data);
    assert_eq!(
        result,
        "0000000000000000000000000000000000000000000000000000000000000037"
    );
}

fn evm_calc_works() {
    // input Calculator.add(7, 2)
    let data = "771602f700000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000002";
    let result = run_revm_calc_contract(data);
    assert_eq!(
        result,
        "0000000000000000000000000000000000000000000000000000000000000009"
    );
}

fn bench_fibb(c: &mut Criterion) {
    c.bench_function("Revm fibb benchmark", |b| b.iter(|| evm_fibb_works()));
}

fn bench_calc(c: &mut Criterion) {
    c.bench_function("Revm calc benchmark", |b| b.iter(|| evm_calc_works()));
}

criterion_group! {
    name = revm_benches;
    config = Criterion::default();
    targets = bench_calc, bench_fibb
}
criterion_main!(revm_benches);
