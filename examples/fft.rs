use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::new(440.0));
    let saw = graph.add(SineOscillator::new(440.0));

    let fft = graph.add(
        FftGraph::new(512, 128, raug::fft::WindowFunction::Hann).build(|fft| {
            let sine_input = fft.add_input();
            let saw_input = fft.add_input();
            let output = fft.add_output();

            // let convolved = sine_input * saw_input;
            let convolved = saw_input;

            output.input(0).connect(convolved.output(0));
        }),
    );

    sine.output(0).connect(&fft.input(0));
    saw.output(0).connect(&fft.input(1));

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
