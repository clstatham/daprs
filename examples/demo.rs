use raug::prelude::*;

fn main() {
    // initialize logging
    env_logger::init();

    // create a new graph
    let graph = GraphBuilder::new();

    // add some outputs
    let out1 = graph.add_output();
    let out2 = graph.add_output();

    // add a sine oscillator
    let sine = graph.add(SineOscillator::new(440.0));

    // set the amplitude of the sine oscillator
    let sine = sine * 0.2;

    // connect the sine oscillator to the outputs
    sine.output(0).connect(&out1.input(0));
    sine.output(0).connect(&out2.input(0));

    // build the graph
    let mut runtime = graph.build_runtime();

    // // run the runtime for 1 second and output to the default audio device
    // runtime
    //     .run_for(Duration::from_secs(1), Backend::Default, Device::Default)
    //     .unwrap();

    // run the graph for 1 second
    runtime
        .run_offline_to_file("target/demo.wav", Duration::from_secs(1), 44_100.0, 512)
        .unwrap();
}
