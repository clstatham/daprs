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
    gain: f64,
    sample_rate: f64,
    envelope: f64,

    /// The threshold amplitude.
    pub threshold: f64,
    /// The attack factor.
    pub attack: f64,
    /// The release factor.
    pub release: f64,
}

impl PeakLimiter {
    /// Creates a new peak limiter processor with the given default threshold, attack coefficient, and release coefficient.
    pub fn new(threshold: f64, attack: f64, release: f64) -> Self {
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

impl Process for PeakLimiter {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::unbounded("in", 0.0),
            SignalSpec::unbounded("threshold", self.threshold),
            SignalSpec::unbounded("attack", self.attack),
            SignalSpec::unbounded("release", self.release),
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
        inputs: ProcessInputs,
        mut outputs: ProcessOutputs,
    ) -> Result<(), ProcessorError> {
        for (out, in_signal, threshold, attack, release) in itertools::izip!(
            outputs.iter_output_mut_as_samples(0)?,
            inputs.iter_input_as_samples(0)?,
            inputs.iter_input_as_samples(1)?,
            inputs.iter_input_as_samples(2)?,
            inputs.iter_input_as_samples(3)?
        ) {
            self.threshold = **threshold;
            self.release = **release;
            self.attack = **attack;

            self.envelope = in_signal.abs().max(self.envelope * self.release);

            let target_gain = if self.envelope > self.threshold {
                self.threshold / self.envelope
            } else {
                1.0
            };

            self.gain = self.gain * self.attack + target_gain * (1.0 - self.attack);

            **out = **in_signal * self.gain;
        }

        Ok(())
    }
}
