use crate::prelude::*;
use crate::proc_fn;

proc_fn!(sine_osc(graph, frequency, amplitude, time) {
    (time * frequency * std::f64::consts::TAU).sin() * amplitude
});

proc_fn!(fm_sine_osc(graph, frequency, amplitude, fm_input, fm_amplitude, time) {
    (time * frequency * std::f64::consts::TAU + fm_input * fm_amplitude).sin() * amplitude
});

proc_fn!(pwm_osc(graph, frequency, amplitude, width, time) {
    let phase = time * frequency % 1.0;

    let pulse = phase.gt(width);

    (pulse * 2.0 - 1.0) * amplitude
});

proc_fn!(saw_osc(graph, frequency, amplitude, time) {
    let phase = time * frequency % 1.0;

    (phase * 2.0 - 1.0) * amplitude
});

proc_fn!(sinc_osc(graph, frequency, time) {
    let phase = time * frequency * std::f64::consts::TAU;

    let zero = graph.ar_constant(0.0);
    let one = graph.ar_constant(1.0);
    let phase = phase.eq(zero).if_else(one, phase);

    let sinc = phase.sin() / phase;

    sinc
});

proc_fn!(bl_saw_osc(graph, frequency) {
    let osc = graph.processor(BlSawOsc::ar());
    osc.connect_inputs([(frequency, 0)]);
    osc
});
