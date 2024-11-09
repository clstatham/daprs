use criterion::{criterion_group, criterion_main, Criterion};
use raug::prelude::*;

const SAMPLE_RATE: u32 = 48_000;
const BUFFER_SIZES: &[usize] = &[32, 128, 256, 512, 1024, 2048, 4096];

fn bench_demo(c: &mut Criterion) {
    let graph = GraphBuilder::new();

    let out1 = graph.add_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").set(440.0);
    let sine = sine * 0.2;
    sine.output(0).connect(&out1.input(0));

    let mut runtime = graph.build_runtime();

    let mut group = c.benchmark_group("demo");

    for &buffer_size in BUFFER_SIZES {
        runtime.reset(SAMPLE_RATE as Sample, buffer_size).unwrap();
        runtime.prepare().unwrap();

        group.throughput(criterion::Throughput::Elements(buffer_size as u64));
        group.bench_function(format!("buffer_size_{}", buffer_size), |b| {
            b.iter(|| {
                runtime.process().unwrap();
            });
        });
    }

    group.finish();
}

fn bench_demo_realtime_simulation(c: &mut Criterion) {
    let graph = GraphBuilder::new();

    let out1 = graph.add_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").set(440.0);
    let sine = sine * 0.2;
    sine.output(0).connect(&out1.input(0));

    let mut runtime = graph.build_runtime();

    let mut group = c.benchmark_group("demo_realtime_simulation");

    for &buffer_size in BUFFER_SIZES {
        runtime.reset(SAMPLE_RATE as Sample, buffer_size).unwrap();
        runtime.prepare().unwrap();

        group.throughput(criterion::Throughput::Elements(buffer_size as u64));
        group.bench_function(format!("buffer_size_{}", buffer_size), |b| {
            b.iter(|| {
                runtime.reset(SAMPLE_RATE as Sample, buffer_size).unwrap();
                runtime.process().unwrap();
            });
        });
    }

    group.finish();
}

fn bench_big_graph(c: &mut Criterion) {
    let graph = GraphBuilder::new();

    let out1 = graph.add_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").set(440.0);
    let mut sine = sine * 0.2;
    for _ in 0..1000 {
        sine = sine * 0.99;
    }
    sine.output(0).connect(&out1.input(0));

    let mut runtime = graph.build_runtime();

    let mut group = c.benchmark_group("big_graph");

    for &buffer_size in BUFFER_SIZES {
        runtime.reset(SAMPLE_RATE as Sample, buffer_size).unwrap();
        runtime.prepare().unwrap();

        group.throughput(criterion::Throughput::Elements(buffer_size as u64));
        group.bench_function(format!("buffer_size_{}", buffer_size), |b| {
            b.iter(|| {
                runtime.process().unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_demo,
    bench_demo_realtime_simulation,
    bench_big_graph
);
criterion_main!(benches);
