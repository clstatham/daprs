use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine1 = graph.add(SineOscillator::new(440.0));
    let sine2 = graph.add(SineOscillator::new(880.0));

    let clock = graph.add(Metro::new(0.5));
    let counter = graph.add(Counter::default());
    counter.input(0).connect(clock.output(0));

    let expr = graph.add(Expr::new("if(counter % 2 < 1, a, b)"));

    expr.input("counter")
        .connect(counter.output(0).cast(SignalType::Float));
    expr.input("a").connect(sine1.output(0));
    expr.input("b").connect(sine2.output(0));

    let sine = expr * 0.2;

    sine.output(0).connect(&out1.input(0));
    sine.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    runtime
        .run_for(
            Duration::from_secs(5),
            AudioBackend::Default,
            AudioDevice::Default,
            None,
        )
        .unwrap();
}
