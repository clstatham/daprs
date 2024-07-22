use std::time::Duration;

use papr::prelude::*;

pub fn mix(inputs: &[Node]) -> Node {
    if inputs.len() == 1 {
        inputs[0].clone()
    } else {
        let mut sum = inputs[0].clone();
        for input in &inputs[1..] {
            sum += input.clone();
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
    out1.connect_inputs([(master.clone(), 0)]);
    out2.connect_inputs([(master, 0)]);

    // create a runtime and run it for 8 seconds
    let graph = graph.build();
    {
        let mut dot = std::fs::File::create("target/demo.dot").unwrap();
        graph.write_dot(&mut dot).unwrap();
    }

    let mut runtime = Runtime::new(graph);

    let out = runtime.run_offline(Duration::from_secs(8), 48_000.0, 48000.0, 512);

    // write the output to a file
    let mut writer = hound::WavWriter::create(
        "target/output.wav",
        hound::WavSpec {
            channels: runtime.graph().num_outputs() as u16,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();

    let num_channels = out.len();
    let num_samples = out[0].len();

    for sample_index in 0..num_samples {
        for channel_index in 0..num_channels {
            writer
                .write_sample(*out[channel_index][sample_index] as f32)
                .unwrap();
        }
    }

    writer.finalize().unwrap();
}
