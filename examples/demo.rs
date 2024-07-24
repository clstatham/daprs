use std::time::Duration;

use papr::prelude::*;

pub fn mix<'g>(inputs: &[Node<'g>]) -> Node<'g> {
    if inputs.len() == 1 {
        inputs[0]
    } else {
        let mut sum = inputs[0];
        for input in &inputs[1..] {
            sum += *input;
        }
        sum
    }
}

fn main() {
    // initialize logging
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("PAPR_LOG")
            .default_filter_or("info"),
    );

    // create some graph nodes
    let graph = Graph::new_builder();
    let out1 = graph.output();
    let out2 = graph.output();
    let time_ar = graph.processor(Time::ar());

    let trigger = pwm_osc(&graph, 2.0, 1.0, 0.01, time_ar);

    let env = graph.processor(DecayEnv::ar());
    env.connect_input(0, trigger, 0);
    env.connect_input(1, 1.0, 0);
    env.connect_input(2, 0.9999, 0);

    let saw1 = bl_saw_osc(&graph, 440.0);

    let master = env * saw1 * 0.1;

    // connect the outputs
    out1.connect_inputs([(master, 0)]);
    out2.connect_inputs([(master, 0)]);

    // create a runtime and run it for 8 seconds
    let graph = graph.build();
    {
        let mut dot = std::fs::File::create("target/demo.dot").unwrap();
        graph.write_dot(&mut dot).unwrap();
    }

    let mut runtime = Runtime::new(graph);

    runtime.run_offline_to_file(
        "target/output.wav",
        Duration::from_secs(8),
        48_000.0,
        48_000.0,
        512,
    );
}
