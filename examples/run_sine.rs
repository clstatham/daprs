use std::time::Duration;

use papr::prelude::*;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().filter("PAPR_LOG"));

    let graph = GraphBuilder::new();
    let out1 = graph.add_output();
    let out2 = graph.add_output();
    let time = graph.add_processor(Time::default());

    let sine_wave = (time * 440.0 * std::f64::consts::TAU).sin() * 0.2;

    out1.connect_inputs([(sine_wave, 0)]);
    out2.connect_inputs([(sine_wave, 0)]);

    let mut runtime = Runtime::new(graph.build());

    runtime.run_for(
        Duration::from_secs(2),
        Backend::Default,
        Device::Name("Realtek HD Audio".to_string()),
    );
}
