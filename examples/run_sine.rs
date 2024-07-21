use pypapr::prelude::*;

fn main() {
    let graph = GraphBuilder::new();
    let time = graph.add_processor(Time::default());

    let sine_wave = (time * 440.0 * std::f64::consts::TAU).sin() * 0.5;

    let out1 = graph.add_output();
    out1.connect_inputs([(sine_wave, 0)]);
    let out2 = graph.add_output();
    out2.connect_inputs([(sine_wave, 0)]);

    let runtime = Runtime::new(graph.build());

    #[cfg(target_os = "linux")]
    let handle = runtime.run(Backend::Alsa);
    #[cfg(target_os = "windows")]
    let handle = runtime.run(Backend::Wasapi);

    std::thread::sleep(std::time::Duration::from_millis(1000));
    handle.stop();
}
