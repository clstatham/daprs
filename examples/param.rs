use raug::prelude::*;

fn main() {
    // initialize logging
    env_logger::init();

    // create a new graph
    let graph = GraphBuilder::new();

    // add some outputs
    let out1 = graph.output();
    let out2 = graph.output();

    // create a sine oscillator
    let sine = graph.sine_osc();

    // create a parameter for the frequency
    // this will allow us to change the frequency of the sine oscillator while the graph is running
    let freq_param = sine.input("frequency").param();
    freq_param.set(440.0);

    // set the amplitude of the sine oscillator
    let sine = sine * 0.2;

    // connect the sine oscillator to the outputs
    sine.output(0).connect(&out1.input(0));
    sine.output(0).connect(&out2.input(0));

    // build the runtime
    let mut runtime = graph.build_runtime();

    // run the graph for 1 second
    let handle = runtime.run(Backend::Default, Device::Default).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));

    // change the frequency of the sine oscillator a few times
    freq_param.set(880.0);
    std::thread::sleep(std::time::Duration::from_secs(1));

    freq_param.set(220.0);
    std::thread::sleep(std::time::Duration::from_secs(1));

    // stop playback
    handle.stop();
}
