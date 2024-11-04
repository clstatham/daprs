//! Oscillator processors.

use crate::prelude::*;

/// A phase accumulator.
///
/// See also: [`GraphBuilder::phase_accum`](crate::builder::graph_builder::GraphBuilder::phase_accum).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhaseAccumulator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Process for PhaseAccumulator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("increment", 0.0),
            SignalSpec::unbounded("reset", Signal::new_message_none()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let increment = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;

        let reset = inputs[1]
            .as_message()
            .ok_or(ProcessorError::InputSpecMismatch(1))?;

        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, increment, reset) in itertools::izip!(out, increment, reset) {
            if reset.is_some() {
                self.t = 0.0;
            }

            // output the phase accumulator value
            **out = self.t;

            // increment the phase accumulator
            self.t_step = **increment;
            self.t += self.t_step;
        }

        Ok(())
    }
}

impl GraphBuilder {
    /// A phase accumulator.
    ///
    /// The phase accumulator is a simple processor that generates a phase signal that increments linearly over time.
    /// It can be used to drive oscillators, or to generate control signals.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `increment` | `Sample` | `0.0` | The phase increment per sample. |
    /// | `1` | `reset` | `Message(Bang)` |  | A message to reset the phase accumulator. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Sample` | The output phase signal. |
    pub fn phase_accum(&self) -> Node {
        self.add_processor(PhaseAccumulator::default())
    }
}

/// A free-running sine wave oscillator.
///
/// See also: [`GraphBuilder::sine_osc`](crate::builder::graph_builder::GraphBuilder::sine_osc).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SineOscillator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
    // sample rate
    sample_rate: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
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

/// A free-running sawtooth wave oscillator.
///
/// See also: [`GraphBuilder::saw_osc`](crate::builder::graph_builder::GraphBuilder::saw_osc).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SawOscillator {
    // phase accumulator
    t: f64,
    // phase increment per sample
    t_step: f64,
    // sample rate
    sample_rate: f64,
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Process for SawOscillator {
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
    /// A free-running sawtooth wave oscillator.
    ///
    /// # Inputs
    ///
    /// | Index | Name | Type | Default | Description |
    /// | --- | --- | --- | --- | --- |
    /// | `0` | `frequency` | `Sample` | `440.0` | The frequency of the sawtooth wave in Hz. |
    /// | `1` | `phase` | `Sample` | `0.0` | The phase of the sawtooth wave in radians. |
    /// | `2` | `reset` | `Message(Bang)` |  | A message to reset the oscillator phase. |
    ///
    /// # Outputs
    ///
    /// | Index | Name | Type | Description |
    /// | --- | --- | --- | --- |
    /// | `0` | `out` | `Sample` | The output sawtooth wave signal. |
    pub fn saw_osc(&self) -> Node {
        self.add_processor(SawOscillator::default())
    }
}
