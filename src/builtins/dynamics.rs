//! Dynamics processors, such as compressors and limiters.

use crate::prelude::*;

/// A peak limiter processor.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Sample` | | The input signal to limit. |
/// | `1` | `threshold` | `Sample` | `~0.99` | The threshold amplitude. |
/// | `2` | `attack` | `Sample` | `0.9` | The attack coefficient. |
/// | `3` | `release` | `Sample` | `0.9995` | The release coefficient. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Sample` | The limited output signal. |
#[derive(Debug, Clone)]
pub struct PeakLimiter {
    gain: Sample,
    sample_rate: Sample,
    envelope: Sample,

    /// The threshold amplitude.
    pub threshold: Sample,
    /// The attack factor.
    pub attack: Sample,
    /// The release factor.
    pub release: Sample,
}

impl PeakLimiter {
    /// Creates a new peak limiter processor with the given default threshold, attack coefficient, and release coefficient.
    pub fn new(threshold: Sample, attack: Sample, release: Sample) -> Self {
        Self {
            threshold,
            attack,
            release,
            ..Default::default()
        }
    }
}

impl Default for PeakLimiter {
    fn default() -> Self {
        Self {
            gain: 1.0,
            sample_rate: 0.0,
            envelope: 0.0,
            // -0.1 dBFS
            threshold: 0.9885530946569389,
            attack: 0.9,
            release: 0.9995,
        }
    }
}

impl Processor for PeakLimiter {
    fn input_names(&self) -> Vec<String> {
        vec![
            String::from("in"),
            String::from("threshold"),
            String::from("attack"),
            String::from("release"),
        ]
    }

    fn output_spec(&self) -> Vec<OutputSpec> {
        vec![OutputSpec::new("out", SignalKind::Sample)]
    }

    fn resize_buffers(&mut self, sample_rate: Sample, _block_size: usize) {
        self.sample_rate = sample_rate;
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal, threshold, attack, release) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_samples(2)?,
            inputs.iter_input_as_samples(3)?
        ) {
            if let Some(threshold) = threshold {
                self.threshold = threshold;
            }

            if let Some(attack) = attack {
                self.attack = attack;
            }

            if let Some(release) = release {
                self.release = release;
            }

            let Some(in_signal) = in_signal else {
                *out = None;
                continue;
            };

            self.envelope = in_signal.abs().max(self.envelope * self.release);

            let target_gain = if self.envelope > self.threshold {
                self.threshold / self.envelope
            } else {
                1.0
            };

            self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

            *out = Some(in_signal * self.gain);
        }

        Ok(())
    }
}
