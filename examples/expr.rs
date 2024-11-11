use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let expr = graph.add(Expr::new("math::sin(2 * pi * t * freq) * 0.2"));

    let pa = graph.add(PhaseAccumulator::default());
    let sr = graph.sample_rate();
    let pi = raug::signal::PI;
    let freq = 440.0;

    pa.input(0).connect(&sr.recip().output(0));

    let t = pa % 1.0;

    expr.input("t").connect(&t.output(0));
    expr.input("freq").set(freq);
    expr.input("pi").set(pi);

    expr.output(0).connect(&out1.input(0));
    expr.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    runtime
        .run_for(
            Duration::from_secs(1),
            AudioBackend::Default,
            AudioDevice::Default,
            MidiPort::Default,
        )
        .unwrap();
}
