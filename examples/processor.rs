use raug::prelude::*;

#[derive(Debug, Clone)]
struct GainProc {
    gain: Sample,
}

impl Processor for GainProc {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("in", 0.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
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
            *output = input * self.gain;
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
            MidiPort::Default,
        )
        .unwrap();
}
