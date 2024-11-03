use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct SineOscillator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
    // sample rate
    sample_rate: f64,
}

impl Process for SineOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("frequency", 440.0),
            SignalSpec::unbounded("phase", 0.0),
            SignalSpec::unbounded("reset", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let phase = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let reset = inputs[2]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(2))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, frequency, phase, reset) in itertools::izip!(out, frequency, phase, reset) {
            if reset.is_some() {
                self.t = 0.0;
            }

            // calculate the sine wave using the phase accumulator
            let sine = (self.t * std::f64::consts::TAU + **phase).sin();
            **out = sine;

            // increment the phase accumulator
            self.t_step = **frequency / self.sample_rate;
            self.t += self.t_step;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A free-running sine wave oscillator.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `frequency` | `Sample` | `440.0` | The frequency of the sine wave in Hz. |
    /// | `1` | `phase` | `Sample` | `0.0` | The phase of the sine wave in radians. |
    /// | `2` | `reset` | `Message(Bang)` |  | A message to reset the oscillator phase. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Sample` | The output sine wave signal. |
    pub fn sine_osc(&self) -> Node {
        self.add_processor(SineOscillator::default())
    }
}

#[derive(Clone, Debug, Default)]
pub struct SawOscillator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
    // sample rate
    sample_rate: f64,
}

impl Process for SawOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("frequency", 440.0),
            SignalSpec::unbounded("phase", 0.0),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let phase = inputs[1]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, frequency, phase) in itertools::izip!(out, frequency, phase) {
            // calculate the sawtooth wave using the phase accumulator
            **out = (self.t + **phase) % 1.0;

            // increment the phase accumulator
            self.t_step = **frequency / self.sample_rate;
            self.t += self.t_step;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A free-running unipolar sawtooth wave oscillator.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `frequency` | `Sample` | `440.0` | The frequency of the sawtooth wave in Hz. |
    /// | `1` | `phase` | `Sample` | `0.0` | The phase of the sawtooth wave in radians. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Sample` | The output unipolar sawtooth wave signal. |
    pub fn saw_osc(&self) -> Node {
        self.add_processor(SawOscillator::default())
    }
}
