use criterion::{criterion_group, criterion_main, Criterion};
use raug::prelude::*;

const SAMPLE_RATE: u32 = 48_000;
const BUFFER_SIZES: &[usize] = &[32, 128, 256, 512, 1024, 2048, 4096];

fn bench_demo(c: &mut Criterion) {
    let graph = GraphBuilder::new();

    let out1 = graph.add_output();

    let sine = graph.sine_osc();
    sine.input("frequency").set(440.0);
    let sine = sine * 0.2;
    sine.output(0).connect(&out1.input(0));

    let mut runtime = graph.build_runtime();

    let mut group = c.benchmark_group("demo");

    for &buffer_size in BUFFER_SIZES {
        runtime.reset(SAMPLE_RATE as f64, buffer_size).unwrap();
        runtime.prepare().unwrap();

        group.throughput(criterion::Throughput::Elements(buffer_size as u64));
        group.bench_function(format!("buffer_size_{}", buffer_size), |b| {
            b.iter(|| {
                criterion::black_box(runtime.next_buffer().unwrap().count());
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_demo);
criterion_main!(benches);
