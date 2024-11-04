use raug::prelude::*;

#[derive(Debug, Clone)]
struct GainProc {
    gain: f64,
}

impl Process for GainProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn process(
        &mut self,
        input: &[SignalBuffer],
        output: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let input = input[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let output = output[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (input, output) in itertools::izip!(input, output) {
            **output = **input * self.gain;
        }
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.output();
    let out2 = graph.output();

    let sine = graph.sine_osc();
    sine.input("frequency").set(440.0);

    let gain = graph.add_processor(GainProc { gain: 0.5 });

    sine.output(0).connect(&gain.input(0));

    gain.output(0).connect(&out1.input(0));
    gain.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    runtime
        .run_for(Duration::from_secs(1), Backend::Default, Device::Default)
        .unwrap();
}
