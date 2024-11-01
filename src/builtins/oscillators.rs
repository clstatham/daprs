use crate::{prelude::*, processor::ProcessorError, signal::SignalBuffer};

/// A free-running sine wave oscillator.
///
/// # Inputs
///
/// | Index | Name | Default | Description |
/// | --- | --- | --- | --- |
/// | `0` | `frequency` | `440.0` | The frequency of the sine wave in Hz. |
///
/// # Outputs
///
/// | Index | Name | Description |
/// | --- | --- | --- |
/// | `0` | `out` | The output sine wave signal. |
#[derive(Clone, Debug)]
pub struct SineOscillator {
    t: f64,
    t_step: f64,
}

impl Default for SineOscillator {
    fn default() -> Self {
        Self {
            t: 0.0,
            t_step: 0.0,
        }
    }
}

impl Process for SineOscillator {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("frequency", 440.0)]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::unbounded("out", 0.0)]
    }

    fn resize_buffers(&mut self, sample_rate: f64, _block_size: usize) {
        self.t_step = sample_rate.recip();
    }

    fn process(
        &mut self,
        inputs: &[SignalBuffer],
        outputs: &mut [SignalBuffer],
    ) -> Result<(), ProcessorError> {
        let frequency = inputs[0]
            .as_sample()
            .ok_or(ProcessorError::InputSpecMismatch(0))?;
        let out = outputs[0]
            .as_sample_mut()
            .ok_or(ProcessorError::OutputSpecMismatch(0))?;

        for (out, frequency) in itertools::izip!(out, frequency) {
            *out = (self.t * **frequency * 2.0 * std::f64::consts::PI)
                .sin()
                .into();
            self.t += self.t_step;
        }

        Ok(())
    }
}
