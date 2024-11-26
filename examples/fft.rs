use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::new(110.0));
    let noise = graph.add(NoiseOscillator::new());
    let noise = noise * 2.0 - 1.0;
    let saw = graph.add(BlSquareOscillator::new(440.0, 0.5));

    let fft = graph.add(FftGraph::new(512, 64, WindowFunction::Hann).build(|fft| {
        let sine_input = fft.add_audio_input();
        let noise_input = fft.add_audio_input();
        let saw_input = fft.add_audio_input();
        let output = fft.add_audio_output();

        let convolved = sine_input * noise_input;

        // let shifter = fft.add(FreqShift::new(2.0));
        // shifter.input(0).connect(saw_input);

        // let shifter = sine_input;

        output.input(0).connect(convolved);
    }));

    sine.output(0).connect(&fft.input(0));
    noise.output(0).connect(&fft.input(1));
    saw.output(0).connect(&fft.input(2));

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
}
