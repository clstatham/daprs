//! Dynamics processors, such as compressors and limiters.

use crate::prelude::*;

/// A peak limiter processor.
///
/// # Inputs
///
/// | Index | Name | Type | Default | Description |
/// | --- | --- | --- | --- | --- |
/// | `0` | `in` | `Float` | | The input signal to limit. |
/// | `1` | `threshold` | `Float` | `~0.99` | The threshold amplitude. |
/// | `2` | `attack` | `Float` | `0.9` | The attack coefficient. |
/// | `3` | `release` | `Float` | `0.9995` | The release coefficient. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Float` | The limited output signal. |
#[derive(Debug, Clone)]
pub struct PeakLimiter {
    gain: Float,
    sample_rate: Float,
    envelope: Float,

    /// The threshold amplitude.
    pub threshold: Float,
    /// The attack factor.
    pub attack: Float,
    /// The release factor.
    pub release: Float,
}

impl PeakLimiter {
    /// Creates a new peak limiter processor with the given default threshold, attack coefficient, and release coefficient.
    pub fn new(threshold: Float, attack: Float, release: Float) -> Self {
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
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", SignalType::Float),
            SignalSpec::new("threshold", SignalType::Float),
            SignalSpec::new("attack", SignalType::Float),
            SignalSpec::new("release", SignalType::Float),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", SignalType::Float)]
    }

    fn resize_buffers(&mut self, sample_rate: Float, _block_size: usize) {
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
