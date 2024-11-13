use benchmarks::{
    geeco_vm::geeco_bench, wasmtime_vm::wasmtime_bench, wasmtime_vm_aot::wasmtime_bench_aot,
};
use criterion::{criterion_group, criterion_main, measurement::Measurement, Criterion};

fn wasmtime_arith<M: Measurement + 'static>(c: &mut Criterion<M>) {
    wasmtime_bench(c, "vc", "arith");
}

fn geeco_arith<M: Measurement + 'static>(c: &mut Criterion<M>) {
    geeco_bench(c, "vc", "arith");
}

fn wasmtime_call<M: Measurement + 'static>(c: &mut Criterion<M>) {
    wasmtime_bench(c, "vc", "call");
}

fn geeco_call<M: Measurement + 'static>(c: &mut Criterion<M>) {
    geeco_bench(c, "vc", "call");
}

fn wasmtime_arith_aot<M: Measurement + 'static>(c: &mut Criterion<M>) {
    wasmtime_bench_aot(c, "vc", "arith");
}

fn wasmtime_call_aot<M: Measurement + 'static>(c: &mut Criterion<M>) {
    wasmtime_bench_aot(c, "vc", "call");
}

criterion_group!(
    name = wasm_benches;
    config = Criterion::default();
    //targets = wasmtime_arith, geeco_arith, wasmtime_arith_aot, wasmtime_call, geeco_call, wasmtime_call_aot
    targets = wasmtime_arith, wasmtime_arith_aot
);
// geeco_arith, wasmtime_call, geeco_call
criterion_main!(wasm_benches);
