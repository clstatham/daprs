use daprs::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add some outputs
    let out1 = graph.output();
    let out2 = graph.output();

    // add a sine oscillator
    let sine = graph.sine_osc();

    // set the frequency of the sine oscillator
    sine.input("frequency").set(440.0);

    // connect the sine oscillator to the outputs
    sine.output(0).connect(out1.input(0));
    sine.output(0).connect(out2.input(0));

    // build the graph
    let mut runtime = graph.build_runtime();

    // // run the runtime for 1 second and output to a file
    // runtime
    //     .run_offline_to_file("target/demo.wav", Duration::from_secs(1), 48_000.0, 512)
    //     .unwrap();

    // run the runtime for 1 second and output to the default audio device
    runtime
        .run_for(Duration::from_secs(1), Backend::Default, Device::Default)
        .unwrap();

    sine.input("frequency").set(880.0);

    let mut runtime = graph.build_runtime();

    // run the runtime for 1 second and output to the default audio device
    runtime
        .run_for(Duration::from_secs(1), Backend::Default, Device::Default)
        .unwrap();
}
