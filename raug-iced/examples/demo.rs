use raug::prelude::*;
use raug_iced::prelude::*;

fn main() {
    // initialize logging
    env_logger::init();

    // create a new graph
    let graph = IcedGraphBuilder::new();

    // add some outputs
    let out1 = graph.output();
    let out2 = graph.output();

    // add a sine oscillator
    let sine = graph.sine_osc();

    // add a frequency knob
    let (freq_knob, freq) = graph.knob();

    // connect the frequency knob to output an audio-type signal, and smooth it
    let freq = freq.m2s().smooth();

    // scale the smooth processor output to a frequency range
    let freq = freq * 1000.0;

    // set the frequency of the sine oscillator
    freq.output(0).connect(sine.input(0));

    // connect the sine oscillator to the outputs
    sine.output(0).connect(out1.input(0));
    sine.output(0).connect(out2.input(0));

    // build the graph and run the runtime
    graph
        .build_runtime(freq_knob)
        .run(Backend::Default, Device::default())
        .unwrap();
}
