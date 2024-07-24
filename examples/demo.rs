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
    let time_kr = graph.processor(Time::kr());

    let freq1 = graph.kr_constant(2.0);
    let amp1 = graph.kr_constant(1.0);
    let width1 = graph.kr_constant(0.01);

    let env = graph.processor(DecayEnv::ar());
    let decay = graph.kr_constant(1.0);
    let curve = graph.kr_constant(0.9999);

    let freq2 = graph.kr_constant(440.0);
    let amp2 = graph.kr_constant(1.0);

    let gain = graph.kr_constant(0.1);

    let trigger = pwm_osc(&graph, freq1.to_ar(), amp1.to_ar(), width1.to_ar(), time_ar);
    env.connect_inputs([(trigger, 0), (decay, 0), (curve, 0)]);

    let saw1 = bl_saw_osc(&graph, freq2.to_ar());

    let master = env * saw1 * gain.to_ar();

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
