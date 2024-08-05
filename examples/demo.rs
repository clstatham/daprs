use std::time::Duration;

use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("PAPR_LOG")
            .default_filter_or("info"),
    );

    // create a new graph
    let graph = GraphBuilder::new();

    // add some outputs
    let out1 = graph.add_output();
    let out2 = graph.add_output();

    // add a processor
    let sine = graph.add(SineOscillator::default());

    // set the frequency of the sine oscillator
    sine.connect_input(440.0, 0, "frequency");

    // connect the processor to the outputs
    sine.connect_output(0, out1, 0);
    sine.connect_output(0, out2, 0);

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 1 second and output to a file
    runtime
        .run_offline_to_file("target/demo.wav", Duration::from_secs(1), 48_000.0, 512)
        .unwrap();
}
