use pypapr::prelude::*;

fn main() {
    let graph = GraphBuilder::new();
    let time = graph.add_processor(Time::default());
    let two_pi = graph.add_processor(Constant::new(std::f64::consts::TAU.into()));
    let freq = graph.add_processor(Constant::new(440.0.into()));

    let gain = graph.add_processor(Constant::new(0.1.into()));

    let sine_wave = (time * freq * two_pi).sin() * gain;

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
