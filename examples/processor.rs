use raug::prelude::*;

#[derive(Debug, Clone)]
struct GainProc {
    gain: Float,
}

impl Processor for GainProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("in", SignalType::Float)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (input, output) in itertools::izip!(
            inputs.iter_input_as_samples(0)?,
            outputs.iter_output_mut_as_samples(0)?
        ) {
            let Some(input) = input else {
                *output = None;
                continue;
            };
            *output = Some(input * self.gain);
        }
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let graph = GraphBuilder::new();

    let out1 = graph.add_audio_output();
    let out2 = graph.add_audio_output();

    let sine = graph.add(SineOscillator::default());
    sine.input("frequency").set(440.0);

    let gain = graph.add(GainProc { gain: 0.5 });

    sine.output(0).connect(&gain.input(0));

    gain.output(0).connect(&out1.input(0));
    gain.output(0).connect(&out2.input(0));

    let mut runtime = graph.build_runtime();

    runtime
        .run_for(
            Duration::from_secs(1),
            AudioBackend::Default,
            AudioDevice::Default,
            None,
        )
        .unwrap();
}
