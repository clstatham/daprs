use raug::prelude::*;

fn main() {
    // initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // create a new graph
    let graph = GraphBuilder::new();

    // add some outputs
    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    // add a buffer reader
    let buf = Buffer::load_wav("examples/assets/piano1.wav").unwrap();
    let len = buf.len() as Sample;
    let buffer = graph.add(AudioBuffer::new(buf));

    // connect the buffer reader to the outputs
    buffer.output(0).connect(&out1.input(0));
    buffer.output(0).connect(&out2.input(0));

    // create a sawtooth oscillator to drive the buffer reader
    let saw = graph.add(SawOscillator::default());

    // we want to read the sample to its full length, so set the frequency to the sample rate divided by the length
    let freq = graph.sample_rate() / len;
    freq.output(0).connect(&saw.input("frequency"));

    // multiply the saw oscillator's amplitude by the length of the buffer, so it outputs the full range of the buffer
    let saw = saw * len;

    // connect the saw oscillator to the buffer reader
    saw.output(0).connect(&buffer.input("position"));

    // build the graph
    let graph = graph.build();

    // create a new runtime
    let mut runtime = Runtime::new(graph);

    // run the runtime for 5 seconds and output to a file
    runtime
        .run_offline_to_file("target/buffer.wav", Duration::from_secs(5), 48_000.0, 512)
        .unwrap();
}
