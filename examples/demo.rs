use std::time::Duration;

use papr::prelude::*;

pub fn sine_osc<'a>(frequency: Node<'a>, amplitude: Node<'a>, time: Node<'a>) -> Node<'a> {
    // sine wave formula: sin(time * frequency * 2 * pi) * amplitude
    (time * frequency * std::f64::consts::TAU).sin() * amplitude
}

pub fn fm_sine_osc<'a>(
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

pub fn pwm_osc<'a>(
    frequency: Node<'a>,
    amplitude: Node<'a>,
    width: Node<'a>,
    time: Node<'a>,
) -> Node<'a> {
    let phase = time * frequency % 1.0;

    let pulse = phase.gt(width);

    (pulse * 2.0 - 1.0) * amplitude
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

    let freq1 = graph.kr_constant(2.0);
    let amp1 = graph.kr_constant(1.0);
    let width1 = graph.kr_constant(0.01);

    let env = graph.processor(DecayEnv::ar());
    let decay = graph.kr_constant(1.0);
    let curve = graph.kr_constant(0.9999);

    let freq2 = graph.kr_constant(440.0);
    let amp2 = graph.kr_constant(1.0);

    let gain = graph.kr_constant(0.5);

    let trigger = pwm_osc(freq1.to_ar(), amp1.to_ar(), width1.to_ar(), time);

    let sine1 = sine_osc(freq2.to_ar(), amp2.to_ar(), time);
    env.connect_inputs([(trigger, 0), (decay, 0), (curve, 0)]);

    let master = env * sine1 * gain.to_ar();

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
