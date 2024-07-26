use crate::prelude::*;
use crate::proc_fn;

proc_fn!(sine_osc(graph, frequency: Control, amplitude: Control, time: Audio) {
    (time * frequency.to_ar() * std::f64::consts::TAU).sin() * amplitude.to_ar()
});

proc_fn!(fm_sine_osc(graph, frequency: Control, amplitude: Control, fm_input: Audio, fm_amplitude: Control, time: Audio) {
    (time * frequency.to_ar() * std::f64::consts::TAU + fm_input * fm_amplitude.to_ar()).sin() * amplitude.to_ar()
});

proc_fn!(pwm_osc(graph, frequency: Control, amplitude: Control, width: Control, time: Audio) {
    let phase = time * frequency.to_ar() % 1.0;

    let pulse = phase.gt(width.to_ar());

    (pulse * 2.0 - 1.0) * amplitude.to_ar()
});

proc_fn!(saw_osc(graph, frequency: Control, amplitude: Control, time: Audio) {
    let phase = time * frequency.to_ar() % 1.0;

    (phase * 2.0 - 1.0) * amplitude.to_ar()
});

proc_fn!(bl_saw_osc(graph, frequency: Control) {
    let osc = graph.processor(BlSawOsc::ar());
    osc.connect_inputs([(frequency.to_ar(), 0)]);
    osc
});
