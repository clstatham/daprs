use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::new(440.0));

    let fft = graph.add(FftSubGraph::build(128, |fft| {
        let input = fft.add_input();
        let output = fft.add_output();
        input.output(0).connect(output.input(0));
    }));

    sine.output(0).connect(&fft.input(0));

    let mix = fft.output(0);

    out1.input(0).connect(&mix);
    out2.input(0).connect(&mix);

    let mut runtime = graph.build_runtime();

    runtime
        .run_offline_to_file("target/fft.wav", Duration::from_secs(5), 48000.0, 512)
        .unwrap();

    // runtime
    //     .run_for(
    //         Duration::from_secs(5),
    //         AudioBackend::Default,
    //         AudioDevice::Default,
    //         None,
    //     )
    //     .unwrap();
}
