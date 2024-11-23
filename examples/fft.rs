use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(BlSawOscillator::new(110.0));
    let saw = graph.add(BlSquareOscillator::new(110.0, 0.5));
    // let saw = saw * 2.0 - 1.0;

    let fft = graph.add(FftGraph::new(512, 256, WindowFunction::Hann).build(|fft| {
        let sine_input = fft.add_input();
        let saw_input = fft.add_input();
        let output = fft.add_output();

        let convolved = sine_input * saw_input;

        output.input(0).connect(convolved.output(0));
    }));

    sine.output(0).connect(&fft.input(0));
    saw.output(0).connect(&fft.input(1));

    let mix = fft.output(0);

    let master = mix;

    out1.input(0).connect(&master);
    out2.input(0).connect(&master);

    let mut runtime = graph.build_runtime();

    runtime
        .run_offline_to_file("target/fft.wav", Duration::from_secs(5), 48000.0, 480)
        .unwrap();

    let wav = hound::WavReader::open("target/fft.wav").unwrap();
    let samples: Vec<f32> = wav.into_samples::<f32>().map(Result::unwrap).collect();
    let max_amp = samples.iter().fold(0.0, |acc, &x| x.abs().max(acc));
    let mean_amp = samples.iter().fold(0.0, |acc, &x| x.abs() + acc) / samples.len() as f32;
    println!("Max amplitude: {}", max_amp);
    println!("Mean amplitude: {}", mean_amp);

    // runtime
    //     .run_for(
    //         Duration::from_secs(5),
    //         AudioBackend::Default,
    //         AudioDevice::Default,
    //         None,
    //     )
    //     .unwrap();
}
