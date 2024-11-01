use std::time::Duration;

use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add a metronome node with an initial period of 0.5 seconds
    let bang = graph.metro(0.5);

    // add a sine oscillator node with a frequency of 1 Hz
    let sine = graph.sine_osc();
    graph.constant(1.0).connect_output(0, sine, "frequency");

    // make the sine only output positive values
    let sine = sine.abs();

    // convert the sine oscillator to output a message
    let sine = sine.s2m();

    // connect the sine oscillator to the metronome
    sine.connect_output(0, bang, "period");

    // add a print node
    let print = graph.print(Some("freq"), None);

    // connect the metronome to trigger the print
    bang.connect_output(0, print, "trig");

    // connect the sine oscillator to the print
    sine.connect_output(0, print, "message");

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 5 seconds
    runtime
        .simulate(Duration::from_secs(5), 44_100.0, 512)
        .unwrap();
}
