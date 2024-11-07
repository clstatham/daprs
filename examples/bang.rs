use raug::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add a metronome node with an initial period of 0.5 seconds
    let bang = graph.add(Metro::new(0.5));

    // add a sine oscillator node with a frequency of 1 Hz
    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").set(1.0);

    // make the sine only output positive values
    let sine = sine.abs();

    // connect the sine oscillator to the metronome
    // sine.connect_output(0, bang, "period");
    sine.output(0).connect(&bang.input("period"));

    // add a print node
    let print = graph.print("freq", None);

    // connect the metronome to trigger the print
    bang.output(0).connect(&print.input("trig"));

    // connect the sine oscillator to the print
    sine.output(0).connect(&print.input("message"));

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 5 seconds
    runtime
        .simulate(Duration::from_secs(5), 44_100.0, 512)
        .unwrap();
}
