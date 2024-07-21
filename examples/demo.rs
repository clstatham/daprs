use std::time::Duration;

use papr::prelude::*;

pub fn sine_wave<'a>(frequency: Node<'a>, amplitude: Node<'a>, time: Node<'a>) -> Node<'a> {
    // sine wave formula: sin(time * frequency * 2 * pi) * amplitude
    (time * frequency * std::f64::consts::TAU).sin() * amplitude
}

pub fn fm_sine_wave<'a>(
    frequency: Node<'a>,
    amplitude: Node<'a>,
    fm_input: Node<'a>,
    fm_amplitude: Node<'a>,
    time: Node<'a>,
) -> Node<'a> {
    (time * frequency * std::f64::consts::TAU + fm_input * fm_amplitude).sin() * amplitude
}

pub fn mix<'a>(inputs: &[Node<'a>]) -> Node<'a> {
    if inputs.len() == 1 {
        inputs[0]
    } else {
        let mut sum = inputs[0];
        for input in &inputs[1..] {
            sum += *input;
        }
        sum
    }
}

fn main() {
    // initialize logging
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("PAPR_LOG")
            .default_filter_or("info"),
    );

    // create some graph nodes
    let graph = Graph::new_builder();
    let out1 = graph.output();
    let out2 = graph.output();
    let time = graph.processor(Time::ar());
    let delay = graph.processor(ConstSampleDelay::ar(4800));

    let freq1 = graph.kr_constant(440.0);
    let freq2 = graph.kr_constant(220.0);
    let amp1 = graph.kr_constant(1.0);
    let amp2 = graph.kr_constant(0.5);
    let gain = graph.kr_constant(0.5);

    let sine1 = sine_wave(freq1.to_ar(), amp1.to_ar(), time);
    delay.connect_input(0, sine1, 0);

    let sine2 = sine_wave(freq2.to_ar(), amp2.to_ar(), time);

    let master = mix(&[delay, sine2]) * gain.to_ar();

    // connect the outputs
    out1.connect_inputs([(master, 0)]);
    out2.connect_inputs([(master, 0)]);

    // create a runtime and run it for 2 seconds
    let mut runtime = Runtime::new(graph.build());
    runtime.run_for(
        Duration::from_secs(2),
        Backend::Default,
        Device::Default,
        480.0,
    );
}
