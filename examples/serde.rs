use raug::prelude::*;

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").connect(440.0);

    let sine = sine * 0.5;

    sine.output(0).connect(&out1.input(0));
    sine.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    let ser = serde_json::to_string_pretty(runtime.graph()).unwrap();
    println!("{}", ser);

    let graph = serde_json::from_str(&ser).unwrap();
    let mut runtime = Runtime::new(graph);

    runtime
        .run_offline_to_file("target/serde.wav", Duration::from_secs(5), 44_100.0, 512)
        .unwrap();
}
