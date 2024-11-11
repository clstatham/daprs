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
    counter.input(0).connect(&clock.output(0));

    let mix = (counter % 2.0).eq(0.0).cond(&sine1, &sine2);

    let mix = mix * 0.2;

    mix.output(0).connect(&out1.input(0));
    mix.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    runtime
        .run_for(
            Duration::from_secs(5),
            AudioBackend::Default,
            AudioDevice::Default,
            MidiPort::Default,
        )
        .unwrap();
}
